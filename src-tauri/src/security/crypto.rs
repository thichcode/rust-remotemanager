use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;
use zeroize::Zeroize;

/// Result type for crypto operations.
pub type Result<T> = std::result::Result<T, CryptoError>;

/// Errors that can occur during cryptographic operations.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    #[error("Invalid key length: {0}")]
    InvalidKeyLength(String),
    #[error("Invalid nonce length: {0}")]
    InvalidNonceLength(String),
}

/// Derive a 32-byte (256-bit) symmetric key from a password using Argon2id.
///
/// ## Parameters
/// * `password` - The user's master password
/// * `salt` - 32-byte random salt
///
/// ## Returns
/// A 32-byte derived key suitable for AES-256-GCM.
///
/// ## Algorithm
/// Uses Argon2id with recommended OWASP parameters:
/// * Time cost: 3 iterations
/// * Memory cost: 64 MiB (65536 KiB)
/// * Parallelism: 4 threads
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(65536, 3, 4, Some(32)).map_err(|e| {
            CryptoError::KeyDerivationFailed(format!("Invalid Argon2 params: {}", e))
        })?,
    );

    let mut output = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    Ok(output)
}

/// Encrypt plaintext using AES-256-GCM.
///
/// ## Parameters
/// * `plaintext` - Bytes to encrypt
/// * `key` - 32-byte AES-256 key
///
/// ## Returns
/// A tuple of `(nonce, ciphertext)` where nonce is 12 bytes.
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<(Vec<u8>, Vec<u8>)> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| {
        CryptoError::EncryptionFailed(format!("Invalid key: {}", e))
    })?;

    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    Ok((nonce_bytes.to_vec(), ciphertext))
}

/// Decrypt ciphertext using AES-256-GCM.
///
/// ## Parameters
/// * `ciphertext` - Encrypted bytes
/// * `nonce` - 12-byte nonce used during encryption
/// * `key` - 32-byte AES-256 key
///
/// ## Returns
/// Decrypted plaintext bytes.
pub fn decrypt(
    ciphertext: &[u8],
    nonce: &[u8],
    key: &[u8; 32],
) -> Result<Vec<u8>> {
    if nonce.len() != 12 {
        return Err(CryptoError::InvalidNonceLength(format!(
            "Expected 12 bytes, got {}",
            nonce.len()
        )));
    }

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| {
        CryptoError::DecryptionFailed(format!("Invalid key: {}", e))
    })?;

    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    Ok(plaintext)
}

/// Derive a sub-key from a master key using HKDF-SHA256 with a context label.
///
/// This allows deriving unique keys for different purposes (e.g., credential encryption,
/// vault header) from a single master key.
///
/// ## Parameters
/// * `master_key` - The 32-byte master key
/// * `context` - Context label bytes (e.g., b"credential" or b"vault-header")
///
/// ## Returns
/// A 32-byte derived sub-key.
pub fn derive_sub_key(master_key: &[u8; 32], context: &[u8]) -> Result<[u8; 32]> {
    let hkdf = Hkdf::<Sha256>::new(Some(context), master_key);
    let mut sub_key = [0u8; 32];
    hkdf.expand(b"subkey", &mut sub_key).map_err(|e| {
        CryptoError::KeyDerivationFailed(format!("HKDF expansion failed: {}", e))
    })?;
    Ok(sub_key)
}

/// Generate a cryptographically secure 32-byte salt for use with Argon2.
pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Generate a cryptographically secure 12-byte nonce for AES-256-GCM.
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Zeroize a byte array to securely clear sensitive data from memory.
pub fn zeroize_bytes(data: &mut [u8]) {
    data.zeroize();
}

/// Zeroize a fixed-size 32-byte array.
pub fn zeroize_key(key: &mut [u8; 32]) {
    key.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        assert_eq!(salt1.len(), 32);
        assert_eq!(salt2.len(), 32);
        assert_ne!(salt1, salt2); // Extremely unlikely to collide
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        assert_eq!(nonce1.len(), 12);
        assert_eq!(nonce2.len(), 12);
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let password = "test_master_password_123!";
        let salt = generate_salt();

        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_different_salts_produce_different_keys() {
        let password = "same_password";
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        let key1 = derive_key(password, &salt1).unwrap();
        let key2 = derive_key(password, &salt2).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let password = "my_secure_password";
        let salt = generate_salt();
        let key = derive_key(password, &salt).unwrap();

        let plaintext = b"Hello, Hermes Remote Manager! This is sensitive data.";
        let (nonce, ciphertext) = encrypt(plaintext, &key).unwrap();

        assert_ne!(ciphertext, plaintext);
        assert_eq!(nonce.len(), 12);

        let decrypted = decrypt(&ciphertext, &nonce, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_empty_data() {
        let password = "password";
        let salt = generate_salt();
        let key = derive_key(password, &salt).unwrap();

        let (nonce, ciphertext) = encrypt(b"", &key).unwrap();
        let decrypted = decrypt(&ciphertext, &nonce, &key).unwrap();
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let password1 = "correct_password";
        let password2 = "wrong_password";
        let salt = generate_salt();

        let key1 = derive_key(password1, &salt).unwrap();
        let key2 = derive_key(password2, &salt).unwrap();

        let plaintext = b"Sensitive data";
        let (nonce, ciphertext) = encrypt(plaintext, &key1).unwrap();

        let result = decrypt(&ciphertext, &nonce, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_sub_key_deterministic() {
        let mut master_key = [0u8; 32];
        OsRng.fill_bytes(&mut master_key);

        let sub1 = derive_sub_key(&master_key, b"credential").unwrap();
        let sub2 = derive_sub_key(&master_key, b"credential").unwrap();

        assert_eq!(sub1, sub2);
    }

    #[test]
    fn test_derive_sub_key_different_context() {
        let mut master_key = [0u8; 32];
        OsRng.fill_bytes(&mut master_key);

        let cred_key = derive_sub_key(&master_key, b"credential").unwrap();
        let vault_key = derive_sub_key(&master_key, b"vault-header").unwrap();

        assert_ne!(cred_key, vault_key);
    }

    #[test]
    fn test_zeroize_key() {
        let mut key = [0xABu8; 32];
        zeroize_key(&mut key);
        assert_eq!(key, [0u8; 32]);
    }

    #[test]
    fn test_invalid_nonce_length() {
        let key = [0u8; 32];
        let result = decrypt(b"ciphertext", b"short", &key);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::InvalidNonceLength(_)));
    }
}
