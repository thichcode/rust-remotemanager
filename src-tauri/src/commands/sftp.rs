use crate::error::{AppError, AppResult};
use crate::ssh::sftp::{SftpFileInfo, SftpSession};
use crate::AppState;
use tauri::State;

/// List directory contents via SFTP for the given session and path.
#[tauri::command]
pub fn list_sftp_dir(
    state: State<AppState>,
    session_id: String,
    path: String,
) -> AppResult<Vec<SftpFileInfo>> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;

    let sftp = SftpSession::new(&ssh_session)?;
    sftp.list_dir(&path)
}

/// Download a remote file via SFTP and return its raw bytes.
#[tauri::command]
pub fn sftp_download(
    state: State<AppState>,
    session_id: String,
    remote_path: String,
) -> AppResult<Vec<u8>> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;

    let sftp = SftpSession::new(&ssh_session)?;
    sftp.read_file(&remote_path)
}

/// Upload data to a remote file via SFTP.
#[tauri::command]
pub fn sftp_upload(
    state: State<AppState>,
    session_id: String,
    remote_path: String,
    data: Vec<u8>,
) -> AppResult<()> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;

    let sftp = SftpSession::new(&ssh_session)?;
    sftp.write_file(&remote_path, &data)
}

/// Create a directory on the remote server via SFTP.
#[tauri::command]
pub fn sftp_mkdir(
    state: State<AppState>,
    session_id: String,
    path: String,
) -> AppResult<()> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;

    let sftp = SftpSession::new(&ssh_session)?;
    sftp.create_dir(&path)
}

/// Remove a file on the remote server via SFTP.
#[tauri::command]
pub fn sftp_rm(
    state: State<AppState>,
    session_id: String,
    path: String,
) -> AppResult<()> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;

    let sftp = SftpSession::new(&ssh_session)?;
    // Determine if it's a file or directory and act accordingly.
    // We try unlink (file) first, then fall back to rmdir (dir).
    match sftp.remove_file(&path) {
        Ok(_) => Ok(()),
        Err(AppError::Ssh(ref msg)) if msg.contains("permission denied") || msg.contains("is a directory") => {
            sftp.remove_dir(&path)
        }
        Err(e) => Err(e),
    }
}

/// Rename a remote file or directory via SFTP.
#[tauri::command]
pub fn sftp_rename(
    state: State<AppState>,
    session_id: String,
    old_path: String,
    new_path: String,
) -> AppResult<()> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;

    let sftp = SftpSession::new(&ssh_session)?;
    sftp.rename(&old_path, &new_path)
}

/// Get file/directory info (stat) via SFTP.
#[tauri::command]
pub fn sftp_stat(
    state: State<AppState>,
    session_id: String,
    path: String,
) -> AppResult<SftpFileInfo> {
    let sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;

    let sftp = SftpSession::new(&ssh_session)?;
    sftp.stat(&path)
}
