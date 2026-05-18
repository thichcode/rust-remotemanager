use crate::AppState;
use crate::error::{AppError, AppResult};
use crate::storage::repositories::credential_repo::CredentialRepository;
use tauri::State;

const PLAINTEXT_PREFIX: &[u8] = b"PLAINTEXT:";

/// Re-encrypt any credentials that were stored as plaintext (when vault was locked).
fn re_encrypt_plaintext_credentials(
    vault: &crate::security::vault::Vault,
    conn: &rusqlite::Connection,
) -> AppResult<()> {
    let repo = CredentialRepository::new(conn);
    let credentials = repo.list()?;
    let mut any_updated = false;

    for cred in &credentials {
        let mut updated = cred.clone();
        let mut changed = false;

        if let Some(ref pw_data) = cred.encrypted_password {
            if pw_data.starts_with(PLAINTEXT_PREFIX) {
                let plaintext = &pw_data[PLAINTEXT_PREFIX.len()..];
                let (nonce, ct) = vault
                    .encrypt_data(plaintext)
                    .map_err(|e| AppError::Vault(format!("Encryption failed: {}", e)))?;
                let mut combined = nonce;
                combined.extend_from_slice(&ct);
                updated.encrypted_password = Some(combined);
                changed = true;
            }
        }

        if let Some(ref pk_data) = cred.encrypted_private_key {
            if pk_data.starts_with(PLAINTEXT_PREFIX) {
                let plaintext = &pk_data[PLAINTEXT_PREFIX.len()..];
                let (nonce, ct) = vault
                    .encrypt_data(plaintext)
                    .map_err(|e| AppError::Vault(format!("Encryption failed: {}", e)))?;
                let mut combined = nonce;
                combined.extend_from_slice(&ct);
                updated.encrypted_private_key = Some(combined);
                changed = true;
            }
        }

        if changed {
            repo.update(updated)?;
            any_updated = true;
        }
    }

    if any_updated {
        tracing::info!("Re-encrypted plaintext credentials after vault unlock");
    }

    Ok(())
}

#[tauri::command]
pub fn vault_status(state: State<AppState>) -> bool {
    let vault = state.vault.lock();
    vault.is_unlocked()
}

#[tauri::command]
pub fn vault_unlock(
    state: State<AppState>,
    password: String,
) -> AppResult<()> {
    let conn = state.db.lock();
    let mut vault = state.vault.lock();
    vault.unlock(&password).map_err(|e| {
        AppError::Vault(format!("Failed to unlock vault: {}", e))
    })?;

    if let Some(salt) = vault.get_salt() {
        let mut settings = state.settings.lock();
        if let Err(e) = settings.set(&conn, "vault_salt", &hex::encode(salt)) {
            tracing::error!("Failed to persist vault salt: {}", e);
        }
    }

    re_encrypt_plaintext_credentials(&*vault, &conn)?;
    Ok(())
}

#[tauri::command]
pub fn vault_lock(state: State<AppState>) -> AppResult<()> {
    let mut vault = state.vault.lock();
    vault.lock();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::vault::Vault;

    #[test]
    fn test_plaintext_prefix() {
        assert_eq!(PLAINTEXT_PREFIX, b"PLAINTEXT:");
    }

    #[test]
    fn test_vault_empty_credentials_no_op() {
        // Create a temp DB with no credentials
        let tmp = tempfile::NamedTempFile::new().unwrap();
        use crate::storage::database::Database;
        use crate::storage::migrations;
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        migrations::run(db.get_conn()).unwrap();

        let mut vault = Vault::new();
        vault.unlock("test").unwrap();
        // Should not error with empty credentials
        let result = re_encrypt_plaintext_credentials(&vault, db.get_conn());
        assert!(result.is_ok());
    }

    #[test]
    fn test_vault_new_is_locked() {
        let vault = Vault::new();
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn test_vault_unlock_and_lock() {
        let mut vault = Vault::new();
        vault.unlock("password").unwrap();
        assert!(vault.is_unlocked());
        vault.lock();
        assert!(!vault.is_unlocked());
    }

    #[test]
    fn test_vault_unlock_twice_fails() {
        let mut vault = Vault::new();
        vault.unlock("pw").unwrap();
        let result = vault.unlock("pw");
        assert!(result.is_err());
    }

    #[test]
    fn test_vault_salt_persistence() {
        let mut vault = Vault::new();
        vault.unlock("password").unwrap();
        let salt = vault.get_salt().unwrap();
        vault.lock();
        assert_eq!(vault.get_salt(), Some(salt));
    }

    #[test]
    fn test_vault_encrypt_decrypt() {
        let mut vault = Vault::new();
        vault.unlock("mypassword").unwrap();
        let (nonce, ct) = vault.encrypt_data(b"secret data").unwrap();
        let decrypted = vault.decrypt_data(&ct, &nonce).unwrap();
        assert_eq!(decrypted, b"secret data");
    }

    #[test]
    fn test_vault_encrypt_when_locked_fails() {
        let vault = Vault::new();
        let result = vault.encrypt_data(b"test");
        assert!(result.is_err());
    }

    #[test]
    fn test_vault_decrypt_when_locked_fails() {
        let vault = Vault::new();
        let result = vault.decrypt_data(b"ct", b"nonce12345678");
        assert!(result.is_err());
    }
}