use crate::AppState;
use crate::error::{AppError, AppResult};
use crate::storage::repositories::credential_repo::CredentialRepository;
use tauri::State;

const PLAINTEXT_PREFIX: &[u8] = b"PLAINTEXT:";

/// Re-encrypt any credentials that were stored as plaintext (when vault was locked).
/// This is called automatically after vault unlock.
fn re_encrypt_plaintext_credentials(
    vault: &crate::security::vault::Vault,
    conn: &rusqlite::Connection,
) -> AppResult<()> {
    let repo = CredentialRepository::new(conn);
    let credentials = repo.list()?;
    let mut has_updates = false;

    for cred in &credentials {
        let mut updated = cred.clone();

        // Re-encrypt password if it starts with PLAINTEXT:
        if let Some(ref pw_data) = cred.encrypted_password {
            if pw_data.starts_with(PLAINTEXT_PREFIX) {
                let plaintext = &pw_data[PLAINTEXT_PREFIX.len()..];
                let (nonce, ct) = vault
                    .encrypt_data(plaintext)
                    .map_err(|e| AppError::Vault(format!("Encryption failed: {}", e)))?;
                let mut combined = nonce;
                combined.extend_from_slice(&ct);
                updated.encrypted_password = Some(combined);
                has_updates = true;
            }
        }

        // Re-encrypt private key if it starts with PLAINTEXT:
        if let Some(ref pk_data) = cred.encrypted_private_key {
            if pk_data.starts_with(PLAINTEXT_PREFIX) {
                let plaintext = &pk_data[PLAINTEXT_PREFIX.len()..];
                let (nonce, ct) = vault
                    .encrypt_data(plaintext)
                    .map_err(|e| AppError::Vault(format!("Encryption failed: {}", e)))?;
                let mut combined = nonce;
                combined.extend_from_slice(&ct);
                updated.encrypted_private_key = Some(combined);
                has_updates = true;
            }
        }

        if has_updates {
            repo.update(updated)?;
        }
    }

    if has_updates {
        tracing::info!("Re-encrypted plaintext credentials after vault unlock");
    }

    Ok(())
}

/// Check whether the vault is currently unlocked.
#[tauri::command]
pub fn vault_status(state: State<AppState>) -> bool {
    let vault = state.vault.lock().unwrap();
    vault.is_unlocked()
}

/// Unlock the vault with the given master password.
/// Persists the salt so re-unlock works after app restart,
/// and re-encrypts any credentials that were stored as plaintext.
#[tauri::command]
pub fn vault_unlock(
    state: State<AppState>,
    password: String,
) -> AppResult<()> {
    let mut vault = state.vault.lock().unwrap();
    vault.unlock(&password).map_err(|e| {
        AppError::Vault(format!("Failed to unlock vault: {}", e))
    })?;

    let conn = state.db.lock().unwrap();

    // Persist salt for re-unlock after restart
    if let Some(salt) = vault.get_salt() {
        let mut settings = state.settings.lock().unwrap();
        settings.set(&conn, "vault_salt", &hex::encode(salt))?;
    }

    // Re-encrypt any plaintext credentials
    re_encrypt_plaintext_credentials(&*vault, &conn)?;

    Ok(())
}

/// Lock the vault, zeroizing the master key in memory.
#[tauri::command]
pub fn vault_lock(state: State<AppState>) -> AppResult<()> {
    let mut vault = state.vault.lock().unwrap();
    vault.lock();
    Ok(())
}
