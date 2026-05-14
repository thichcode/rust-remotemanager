use crate::security::crypto;
use zeroize::Zeroize;

/// Result type for vault operations.
pub type Result<T> = std::result::Result<T, VaultError>;

/// Errors that can occur during vault operations.
#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("Vault is locked")]
    VaultLocked,
    #[error("Vault is already unlocked")]
    VaultAlreadyUnlocked,
    #[error("Crypto error: {0}")]
    CryptoError(#[from] crypto::CryptoError),
}

/// A secure in-memory vault that holds the master encryption key.
///
/// The vault is unlocked by providing a master password, which derives
/// an AES-256 key using Argon2id. While unlocked, it can encrypt and
/// decrypt arbitrary data (typically credential secrets) using
/// context-specific sub-keys derived via HKDF-SHA256.
///
/// ## Security Properties
/// - Master key is stored only in memory while unlocked
/// - `lock()` zeroizes the key before dropping
/// - Each data type uses a different HKDF context (sub-key isolation)
/// - Sub-keys prevent cross-context decryption even with the same vault
pub struct Vault {
    unlocked: bool,
    master_key: Option<[u8; 32]>,
    salt: Option<[u8; 32]>,
}

impl Vault {
    /// Create a new vault in locked state.
    pub fn new() -> Self {
        Self {
            unlocked: false,
            master_key: None,
            salt: None,
        }
    }

    /// Unlock the vault by deriving a key from the master password.
    ///
    /// The salt is randomly generated on first unlock and should be stored
    /// persistently (e.g., in the vault file header) to allow re-unlocking.
    ///
    /// If the vault has been unlocked before, the stored salt is reused.
    /// Otherwise, a new salt is generated.
    pub fn unlock(&mut self, master_password: &str) -> Result<()> {
        if self.unlocked {
            return Err(VaultError::VaultAlreadyUnlocked);
        }

        let salt = self.salt.unwrap_or_else(crypto::generate_salt);
        let key = crypto::derive_key(master_password, &salt)?;

        self.master_key = Some(key);
        self.salt = Some(salt);
        self.unlocked = true;

        Ok(())
    }

    /// Lock the vault, zeroizing the master key in memory.
    pub fn lock(&mut self) {
        if let Some(ref mut key) = self.master_key {
            key.zeroize();
        }
        self.master_key = None;
        self.unlocked = false;
        // Keep the salt so we can re-unlock without regenerating
    }

    /// Check whether the vault is currently unlocked.
    pub fn is_unlocked(&self) -> bool {
        self.unlocked
    }

    /// Get the vault's salt (for persistent storage).
    pub fn get_salt(&self) -> Option<[u8; 32]> {
        self.salt
    }

    /// Set the vault's salt (when loading from persistent storage).
    pub fn set_salt(&mut self, salt: [u8; 32]) {
        self.salt = Some(salt);
    }

    /// Encrypt data using a sub-key derived from the master key with
    /// the "credential" context.
    ///
    /// Returns `(nonce, ciphertext)`.
    ///
    /// # Panics
    /// Panics if vault is not unlocked.
    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        if !self.unlocked {
            return Err(VaultError::VaultLocked);
        }

        let master_key = self.master_key.as_ref().unwrap();
        let sub_key = crypto::derive_sub_key(master_key, b"credential")?;
        let (nonce, ciphertext) = crypto::encrypt(plaintext, &sub_key)?;

        Ok((nonce, ciphertext))
    }

    /// Decrypt data using a sub-key derived from the master key with
    /// the "credential" context.
    ///
    /// # Panics
    /// Panics if vault is not unlocked.
    pub fn decrypt_data(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
        if !self.unlocked {
            return Err(VaultError::VaultLocked);
        }

        let master_key = self.master_key.as_ref().unwrap();
        let sub_key = crypto::derive_sub_key(master_key, b"credential")?;
        let plaintext = crypto::decrypt(ciphertext, nonce, &sub_key)?;

        Ok(plaintext)
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        self.lock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vault_is_locked() {
        let vault = Vault::new();
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn test_unlock_and_lock() {
        let mut vault = Vault::new();
        vault.unlock("master_password").unwrap();
        assert!(vault.is_unlocked());

        vault.lock();
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn test_unlock_twice_fails() {
        let mut vault = Vault::new();
        vault.unlock("password").unwrap();
        let result = vault.unlock("password");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VaultError::VaultAlreadyUnlocked));
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let mut vault = Vault::new();
        vault.unlock("my_secure_password").unwrap();

        let plaintext = b"username=admin&password=supersecret";
        let (nonce, ciphertext) = vault.encrypt_data(plaintext).unwrap();

        assert_ne!(ciphertext, plaintext);

        let decrypted = vault.decrypt_data(&ciphertext, &nonce).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_when_locked_fails() {
        let vault = Vault::new();
        let result = vault.encrypt_data(b"test");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VaultError::VaultLocked));
    }

    #[test]
    fn test_decrypt_when_locked_fails() {
        let vault = Vault::new();
        let result = vault.decrypt_data(b"ciphertext", b"nonce12345678");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VaultError::VaultLocked));
    }

    #[test]
    fn test_salt_persistence_across_lock_cycles() {
        let mut vault = Vault::new();
        vault.unlock("password").unwrap();
        let salt = vault.get_salt().unwrap();

        vault.lock();
        assert_eq!(vault.get_salt(), Some(salt));

        // Re-unlock with the same salt
        vault.unlock("password").unwrap();
        let salt2 = vault.get_salt().unwrap();
        assert_eq!(salt, salt2);
    }

    #[test]
    fn test_set_salt() {
        let mut vault = Vault::new();
        let salt = [0xABu8; 32];
        vault.set_salt(salt);
        assert_eq!(vault.get_salt(), Some(salt));
    }

    #[test]
    fn test_different_passwords_produce_different_encryption() {
        let mut vault1 = Vault::new();
        vault1.unlock("password1").unwrap();

        let mut vault2 = Vault::new();
        vault2.unlock("password2").unwrap();

        let plaintext = b"same data";
        let (nonce1, ct1) = vault1.encrypt_data(plaintext).unwrap();
        let (_, ct2) = vault2.encrypt_data(plaintext).unwrap();

        // Different keys produce different ciphertexts
        assert_ne!(ct1, ct2);

        // Cannot decrypt vault1's data with vault2's key
        let result = vault2.decrypt_data(&ct1, &nonce1);
        assert!(result.is_err());
    }
}
