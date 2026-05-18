use rfd::FileDialog;

use crate::AppState;
use crate::error::{AppError, AppResult};
use crate::storage::models::Credential;
use crate::storage::repositories::credential_repo::CredentialRepository;
use tauri::State;
use uuid::Uuid;

/// Open a native file dialog to pick an SSH private key file. Returns the selected file path.
#[tauri::command]
pub fn pick_ssh_key_file() -> AppResult<String> {
    let path = FileDialog::new()
        .add_filter("SSH Private Key", &["pem", "key", "pub", "rsa", "ed25519", "ppk"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .ok_or_else(|| AppError::Dialog("No file selected".into()))?;
    Ok(path.to_string_lossy().to_string())
}

/// List all stored credentials (encrypted blobs are returned as-is).
#[tauri::command]
pub fn list_credentials(state: State<AppState>) -> AppResult<Vec<Credential>> {
    let db = state.db.lock();
    let repo = CredentialRepository::new(&db);
    repo.list()
}

/// Save a credential. Encrypts password/private key via the vault if unlocked.
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
    let db = state.db.lock();
    let vault = state.vault.lock();

    let encrypted_password = if let Some(pw) = password {
        if vault.is_unlocked() {
            let (nonce, ct) = vault.encrypt_data(pw.as_bytes()).map_err(|e| {
                AppError::Vault(format!("Encryption failed: {}", e))
            })?;
            let mut combined = nonce.clone();
            combined.extend_from_slice(&ct);
            Some(combined)
        } else {
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
    repo.create(credential)
}

/// Delete a credential by ID.
#[tauri::command]
pub fn delete_credential(
    state: State<AppState>,
    id: String,
) -> AppResult<()> {
    let db = state.db.lock();
    let repo = CredentialRepository::new(&db);
    repo.delete(&id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;
    use crate::storage::migrations;
    use tempfile::NamedTempFile;

    fn setup_repo() -> (CredentialRepository<'static>, Box<dyn std::any::Any>) {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let db = Database::new(&path).unwrap();
        let conn = db.into_connection();
        migrations::run(&conn).unwrap();
        let conn: &'static rusqlite::Connection = Box::leak(Box::new(conn));
        let repo = CredentialRepository::new(conn);
        (repo, Box::new(tmp))
    }

    #[test]
    fn test_list_empty() {
        let (repo, _guard) = setup_repo();
        let list = repo.list().unwrap();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_create_password_credential() {
        let (repo, _guard) = setup_repo();
        let cred = repo.create(Credential {
            id: String::new(), name: "My Password".into(), auth_type: "password".into(),
            username: Some("admin".into()), encrypted_password: Some(vec![1,2,3]),
            key_type: None, encrypted_private_key: None, key_path: None,
            passphrase_protected: false, created_at: String::new(), updated_at: String::new(),
        }).unwrap();
        assert_eq!(cred.name, "My Password");
        assert!(!cred.id.is_empty());
    }

    #[test]
    fn test_create_key_credential() {
        let (repo, _guard) = setup_repo();
        let cred = repo.create(Credential {
            id: String::new(), name: "SSH Key".into(), auth_type: "key".into(),
            username: Some("ubuntu".into()), encrypted_password: None,
            key_type: Some("ed25519".into()), encrypted_private_key: Some(vec![4,5,6]),
            key_path: Some("/home/user/.ssh/id_ed25519".into()),
            passphrase_protected: true, created_at: String::new(), updated_at: String::new(),
        }).unwrap();
        assert_eq!(cred.auth_type, "key");
        assert!(cred.key_path.is_some());
    }

    #[test]
    fn test_get_by_id() {
        let (repo, _guard) = setup_repo();
        let cred = repo.create(Credential {
            id: String::new(), name: "Test".into(), auth_type: "password".into(),
            username: Some("test".into()), encrypted_password: Some(vec![10,20]),
            key_type: None, encrypted_private_key: None, key_path: None,
            passphrase_protected: false, created_at: String::new(), updated_at: String::new(),
        }).unwrap();
        let found = repo.get_by_id(&cred.id).unwrap().unwrap();
        assert_eq!(found.name, "Test");
    }

    #[test]
    fn test_update_credential() {
        let (repo, _guard) = setup_repo();
        let mut cred = repo.create(Credential {
            id: String::new(), name: "Original".into(), auth_type: "password".into(),
            username: None, encrypted_password: None, key_type: None,
            encrypted_private_key: None, key_path: None,
            passphrase_protected: false, created_at: String::new(), updated_at: String::new(),
        }).unwrap();
        cred.name = "Updated".into();
        assert!(repo.update(cred.clone()).unwrap());
        let found = repo.get_by_id(&cred.id).unwrap().unwrap();
        assert_eq!(found.name, "Updated");
    }

    #[test]
    fn test_delete_credential() {
        let (repo, _guard) = setup_repo();
        let cred = repo.create(Credential {
            id: String::new(), name: "To Delete".into(), auth_type: "password".into(),
            username: None, encrypted_password: None, key_type: None,
            encrypted_private_key: None, key_path: None,
            passphrase_protected: false, created_at: String::new(), updated_at: String::new(),
        }).unwrap();
        assert!(repo.delete(&cred.id).unwrap());
        assert!(repo.get_by_id(&cred.id).unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let (repo, _guard) = setup_repo();
        assert!(!repo.delete("ghost").unwrap());
    }

    #[test]
    fn test_get_by_id_not_found() {
        let (repo, _guard) = setup_repo();
        let result = repo.get_by_id("nonexistent").unwrap();
        assert!(result.is_none());
    }
}