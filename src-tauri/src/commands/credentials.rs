use crate::AppState;
use crate::error::{AppError, AppResult};
use crate::storage::models::Credential;
use crate::storage::repositories::credential_repo::CredentialRepository;
use tauri::State;
use uuid::Uuid;

/// Open a file dialog to pick an SSH private key file. Returns the selected file path.
#[tauri::command]
pub fn pick_ssh_key_file() -> AppResult<String> {
    let path = tauri::api::dialog::open(None::<&str>, None::<&str>, false)
        .ok_or_else(|| AppError::Dialog("No file selected".into()))?;
    Ok(path)
}

/// List all stored credentials (encrypted blobs are returned as-is).
#[tauri::command]
pub fn list_credentials(state: State<AppState>) -> AppResult<Vec<Credential>> {
    let db = state.db.lock().unwrap();
    let repo = CredentialRepository::new(&db);
    Ok(repo.list()?)
}

/// Save a credential. Encrypts password/private key via the vault if unlocked.
/// If the vault is locked, the credential is saved with raw plaintext data
/// (which will be encrypted on next vault unlock — implement lazy encryption as needed).
#[tauri::command]
pub fn save_credential(
    state: State<AppState>,
    name: String,
    auth_type: String,
    username: Option<String>,
    password: Option<String>,
    private_key: Option<String>,
    key_path: Option<String>,
    passphrase_protected: Option<bool>,
) -> AppResult<Credential> {
    let db = state.db.lock().unwrap();
    let vault = state.vault.lock().unwrap();

    let encrypted_password = if let Some(pw) = password {
        if vault.is_unlocked() {
            let (nonce, ct) = vault.encrypt_data(pw.as_bytes()).map_err(|e| {
                AppError::Vault(format!("Encryption failed: {}", e))
            })?;
            let mut combined = nonce.clone();
            combined.extend_from_slice(&ct);
            Some(combined)
        } else {
            // Store as raw bytes tagged with PLAINTEXT marker
            let mut buf = b"PLAINTEXT:".to_vec();
            buf.extend_from_slice(pw.as_bytes());
            Some(buf)
        }
    } else {
        None
    };

    let encrypted_private_key = if let Some(pk) = private_key {
        if vault.is_unlocked() {
            let (nonce, ct) = vault.encrypt_data(pk.as_bytes()).map_err(|e| {
                AppError::Vault(format!("Encryption failed: {}", e))
            })?;
            let mut combined = nonce.clone();
            combined.extend_from_slice(&ct);
            Some(combined)
        } else {
            let mut buf = b"PLAINTEXT:".to_vec();
            buf.extend_from_slice(pk.as_bytes());
            Some(buf)
        }
    } else {
        None
    };

    let now = chrono::Utc::now().to_rfc3339();
    let credential = Credential {
        id: Uuid::new_v4().to_string(),
        name,
        auth_type,
        username,
        encrypted_password,
        key_type: None,
        encrypted_private_key,
        key_path,
        passphrase_protected: passphrase_protected.unwrap_or(false),
        created_at: now.clone(),
        updated_at: now,
    };

    let repo = CredentialRepository::new(&db);
    Ok(repo.create(credential)?)
}

/// Delete a credential by ID.
#[tauri::command]
pub fn delete_credential(
    state: State<AppState>,
    id: String,
) -> AppResult<()> {
    let db = state.db.lock().unwrap();
    let repo = CredentialRepository::new(&db);
    repo.delete(&id)?;
    Ok(())
}
