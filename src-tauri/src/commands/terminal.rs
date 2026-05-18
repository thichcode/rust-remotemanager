use crate::error::{AppError, AppResult};
use crate::security::vault::Vault;
use crate::ssh::session::SshConfig;
use crate::storage::repositories::credential_repo::CredentialRepository;
use crate::AppState;
use rusqlite::Connection;
use tauri::State;
use uuid::Uuid;

const PLAINTEXT_PREFIX: &[u8] = b"PLAINTEXT:";

/// Resolve credential secrets from the database and inject them into the config.
fn resolve_credential(
    config: &mut serde_json::Map<String, serde_json::Value>,
    conn: &Connection,
    vault: &Vault,
) -> AppResult<()> {
    let credential_id = match config.get("credential_id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id.to_string(),
        _ => return Ok(()),
    };

    let repo = CredentialRepository::new(conn);
    let cred = match repo.get_by_id(&credential_id)? {
        Some(c) => c,
        None => {
            tracing::warn!("[connect_ssh] credential '{}' not found", credential_id);
            return Ok(());
        }
    };

    // Decrypt password if available
    if let Some(ref data) = cred.encrypted_password {
        let plaintext = if data.starts_with(PLAINTEXT_PREFIX) {
            String::from_utf8_lossy(&data[PLAINTEXT_PREFIX.len()..]).to_string()
        } else if data.len() > 12 && vault.is_unlocked() {
            vault
                .decrypt_data(&data[12..], &data[..12])
                .ok()
                .map(|v| String::from_utf8_lossy(&v).to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        if !plaintext.is_empty() {
            config.insert("password".into(), serde_json::Value::String(plaintext));
        }
    }

    // Use key_path from credential if present
    if let Some(ref kp) = cred.key_path {
        config.insert("key_path".into(), serde_json::Value::String(kp.clone()));
    }

    Ok(())
}

/// Connect to a remote host via SSH and create a terminal session.
/// The connection runs on a background thread and streams output via
/// `terminal:output` / `terminal:stderr` / `terminal:error` / `terminal:exit`
/// Tauri events.
#[tauri::command]
pub fn connect_ssh(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    config: serde_json::Value,
) -> AppResult<String> {
    let mut config = config;

    // Resolve credential secrets if credential_id is provided
    if let Some(ref mut obj) = config.as_object_mut() {
        let conn = state.db.lock().map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;
        let vault = state.vault.lock().map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;
        resolve_credential(obj, &conn, &vault)?;
    }

    tracing::debug!("[connect_ssh] config after credential resolution: {}", config);
    let config_val = config.clone();
    let config: SshConfig = serde_json::from_value(config).map_err(|e| {
        AppError::Validation(format!("failed to parse config: {} — input: {}", e, config_val))
    })?;
    tracing::info!(
        "[connect_ssh] parsed: host={}, port={}, username={}, auth_type={}",
        config.host, config.port, config.username, config.auth_type
    );
    let session_id = Uuid::new_v4().to_string();

    let mut sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    sessions.connect(config, app_handle, session_id.clone());
    Ok(session_id)
}

/// Disconnect and remove a terminal session by ID.
#[tauri::command]
pub fn disconnect_session(state: State<AppState>, id: String) -> AppResult<()> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    if sessions.get(&id).is_none() {
        return Err(AppError::NotFound(format!("Session '{}' not found", id)));
    }
    sessions.remove(&id);
    Ok(())
}

/// Send input data to an active terminal session.
#[tauri::command]
pub fn terminal_input(state: State<AppState>, id: String, data: String) -> AppResult<()> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    sessions
        .send_input(&id, &data)
        .map_err(|e| {
            if e.contains("not found") {
                AppError::NotFound(e)
            } else {
                AppError::Ssh(e)
            }
        })
}

/// Resize the terminal PTY for an active session.
#[tauri::command]
pub fn terminal_resize(
    state: State<AppState>,
    id: String,
    cols: u16,
    rows: u16,
) -> AppResult<()> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    sessions.resize(&id, cols, rows).map_err(|e| {
        if e.contains("not found") {
            AppError::NotFound(e)
        } else {
            AppError::Ssh(e)
        }
    })
}

/// List all active session IDs.
#[tauri::command]
pub fn list_sessions(state: State<AppState>) -> Vec<String> {
    let sessions = state.sessions.lock().unwrap();
    sessions.list()
}

/// Get the state of a specific session as a simple string.
#[tauri::command]
pub fn get_session_state(state: State<AppState>, id: String) -> AppResult<Option<String>> {
    let sessions = state.sessions.lock().unwrap();
    Ok(sessions.get_state(&id).map(|s| s.to_simple_string()))
}
