use crate::AppState;
use crate::error::{AppError, AppResult};
use crate::ssh::session::SshConfig;
use tauri::State;
use uuid::Uuid;

/// Connect to a remote host via SSH and create a terminal session.
/// Returns the session ID on success.
#[tauri::command]
pub fn connect_ssh(
    state: State<AppState>,
    config: SshConfig,
) -> AppResult<String> {
    let session_id = Uuid::new_v4().to_string();
    let sessions = state.sessions.lock().unwrap();
    sessions.create(session_id.clone(), config);
    Ok(session_id)
}

/// Disconnect and remove a terminal session by ID.
#[tauri::command]
pub fn disconnect_session(
    state: State<AppState>,
    id: String,
) -> AppResult<()> {
    let sessions = state.sessions.lock().unwrap();
    if sessions.get(&id).is_none() {
        return Err(AppError::NotFound(format!("Session '{}' not found", id)));
    }
    sessions.remove(&id);
    Ok(())
}

/// Send input data to an active terminal session.
#[tauri::command]
pub fn terminal_input(
    state: State<AppState>,
    id: String,
    data: String,
) -> AppResult<()> {
    let sessions = state.sessions.lock().unwrap();
    if sessions.get(&id).is_none() {
        return Err(AppError::NotFound(format!("Session '{}' not found", id)));
    }
    // TODO: Forward data to the actual SSH channel when fully implemented
    tracing::debug!("Terminal input for session {}: {:?}", id, data);
    Ok(())
}

/// Resize the terminal PTY for an active session.
#[tauri::command]
pub fn terminal_resize(
    state: State<AppState>,
    id: String,
    cols: u16,
    rows: u16,
) -> AppResult<()> {
    let sessions = state.sessions.lock().unwrap();
    if sessions.get(&id).is_none() {
        return Err(AppError::NotFound(format!("Session '{}' not found", id)));
    }
    // TODO: Forward resize event to the actual SSH channel when fully implemented
    tracing::debug!("Terminal resize for session {}: {}x{}", id, cols, rows);
    Ok(())
}

/// List all active session IDs.
#[tauri::command]
pub fn list_sessions(state: State<AppState>) -> Vec<String> {
    let sessions = state.sessions.lock().unwrap();
    sessions.list()
}
