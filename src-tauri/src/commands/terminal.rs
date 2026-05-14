use crate::error::{AppError, AppResult};
use crate::ssh::session::SshConfig;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

/// Connect to a remote host via SSH and create a terminal session.
/// The connection runs on a background thread and streams output via
/// `terminal:output` / `terminal:stderr` / `terminal:error` / `terminal:exit`
/// Tauri events.
#[tauri::command]
pub fn connect_ssh(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    config: SshConfig,
) -> AppResult<String> {
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

/// Get the state of a specific session.
#[tauri::command]
pub fn get_session_state(state: State<AppState>, id: String) -> AppResult<Option<crate::ssh::session::SessionState>> {
    let sessions = state.sessions.lock().unwrap();
    Ok(sessions.get_state(&id))
}
