use crate::error::{AppError, AppResult};
use crate::ssh::sftp::{SftpFileInfo, SftpSession};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn list_sftp_dir(
    state: State<AppState>,
    session_id: String,
    path: String,
) -> AppResult<Vec<SftpFileInfo>> {
    let sessions = state.sessions.lock();
    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;
    let sftp = SftpSession::new(&ssh_session)?;
    sftp.list_dir(&path)
}

#[tauri::command]
pub fn sftp_download(
    state: State<AppState>,
    session_id: String,
    remote_path: String,
) -> AppResult<Vec<u8>> {
    let sessions = state.sessions.lock();
    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;
    let sftp = SftpSession::new(&ssh_session)?;
    sftp.read_file(&remote_path)
}

#[tauri::command]
pub fn sftp_upload(
    state: State<AppState>,
    session_id: String,
    remote_path: String,
    data: Vec<u8>,
) -> AppResult<()> {
    let sessions = state.sessions.lock();
    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;
    let sftp = SftpSession::new(&ssh_session)?;
    sftp.write_file(&remote_path, &data)
}

#[tauri::command]
pub fn sftp_mkdir(
    state: State<AppState>,
    session_id: String,
    path: String,
) -> AppResult<()> {
    let sessions = state.sessions.lock();
    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;
    let sftp = SftpSession::new(&ssh_session)?;
    sftp.create_dir(&path)
}

#[tauri::command]
pub fn sftp_rm(
    state: State<AppState>,
    session_id: String,
    path: String,
) -> AppResult<()> {
    let sessions = state.sessions.lock();
    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;
    let sftp = SftpSession::new(&ssh_session)?;
    match sftp.remove_file(&path) {
        Ok(_) => Ok(()),
        Err(AppError::Ssh(ref msg)) if msg.contains("permission denied") || msg.contains("is a directory") => {
            sftp.remove_dir(&path)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub fn sftp_rename(
    state: State<AppState>,
    session_id: String,
    old_path: String,
    new_path: String,
) -> AppResult<()> {
    let sessions = state.sessions.lock();
    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;
    let sftp = SftpSession::new(&ssh_session)?;
    sftp.rename(&old_path, &new_path)
}

#[tauri::command]
pub fn sftp_stat(
    state: State<AppState>,
    session_id: String,
    path: String,
) -> AppResult<SftpFileInfo> {
    let sessions = state.sessions.lock();
    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;
    let sftp = SftpSession::new(&ssh_session)?;
    sftp.stat(&path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::sftp::SftpFileInfo;

    #[test]
    fn test_sftp_file_info_serde_roundtrip() {
        let info = SftpFileInfo {
            name: "test.txt".into(), path: "/home/test.txt".into(),
            size: 1024, is_dir: false, is_symlink: false,
            permissions: "644".into(), modified: "2024-01-01 12:00:00".into(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: SftpFileInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test.txt");
        assert_eq!(deserialized.size, 1024);
    }

    #[test]
    fn test_sftp_file_info_is_dir() {
        let info = SftpFileInfo {
            name: "directory".into(), path: "/home/dir".into(),
            size: 4096, is_dir: true, is_symlink: false,
            permissions: "755".into(), modified: "".into(),
        };
        assert!(info.is_dir);
        assert!(!info.is_symlink);
    }

    #[test]
    fn test_sftp_file_info_is_symlink() {
        let info = SftpFileInfo {
            name: "link".into(), path: "/home/link".into(),
            size: 0, is_dir: false, is_symlink: true,
            permissions: "777".into(), modified: "".into(),
        };
        assert!(info.is_symlink);
    }

    #[test]
    fn test_sftp_file_info_zero_size() {
        let info = SftpFileInfo {
            name: "empty".into(), path: "/empty".into(),
            size: 0, is_dir: false, is_symlink: false,
            permissions: "644".into(), modified: "".into(),
        };
        assert_eq!(info.size, 0);
    }

    #[test]
    fn test_sftp_file_info_permissions_format() {
        let info = SftpFileInfo {
            name: "config".into(), path: "/etc/config".into(),
            size: 512, is_dir: false, is_symlink: false,
            permissions: "644".into(), modified: "".into(),
        };
        assert_eq!(info.permissions, "644");
    }

    #[test]
    fn test_sftp_file_info_path() {
        let info = SftpFileInfo {
            name: "file.log".into(), path: "/var/log/file.log".into(),
            size: 9999, is_dir: false, is_symlink: false,
            permissions: "600".into(), modified: "".into(),
        };
        assert_eq!(info.path, "/var/log/file.log");
    }

    #[test]
    fn test_app_error_not_found_format() {
        let err = AppError::NotFound("Session 'x' not found or not connected".into());
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_sftp_rm_error_message_patterns() {
        // Test the error patterns used in sftp_rm fallback logic
        let perm_err = AppError::Ssh("permission denied".into());
        assert!(perm_err.to_string().contains("permission denied"));
        let dir_err = AppError::Ssh("is a directory".into());
        assert!(dir_err.to_string().contains("is a directory"));
    }
}