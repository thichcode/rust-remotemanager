use crate::error::{AppError, AppResult};
use crate::ssh::session::SessionManager;
use crate::ssh::tunnels::{TunnelConfig, TunnelManager};
use crate::AppState;
use tauri::State;
use uuid::Uuid;

/// List all tunnels for a given session (or all tunnels if session_id is empty).
#[tauri::command]
pub fn list_tunnels(state: State<AppState>, session_id: Option<String>) -> AppResult<Vec<TunnelConfig>> {
    let _sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    let tunnels = state
        .tunnels
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    let all = tunnels.list();
    if let Some(sid) = session_id {
        Ok(all.into_iter().filter(|t| t.session_id == sid).collect())
    } else {
        Ok(all)
    }
}

/// Create a new tunnel. The `session_id` must refer to an active SSH session.
#[tauri::command]
pub fn create_tunnel(
    state: State<AppState>,
    config: TunnelConfig,
    session_id: String,
) -> AppResult<TunnelConfig> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    // Look up the SSH session and get a clone of the raw ssh2::Session
    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;

    let tunnels = state
        .tunnels
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    let mut config = config;
    config.id = Uuid::new_v4().to_string();
    config.session_id = session_id.clone();
    config.active = true;

    let result = match config.tunnel_type {
        crate::ssh::tunnels::TunnelType::Local => tunnels.add_local(&ssh_session, config),
        crate::ssh::tunnels::TunnelType::Remote => tunnels.add_remote(&ssh_session, config),
        crate::ssh::tunnels::TunnelType::Dynamic => tunnels.add_dynamic(&ssh_session, config),
    };

    result
}

/// Stop and remove a tunnel by ID.
#[tauri::command]
pub fn stop_tunnel(state: State<AppState>, id: String) -> AppResult<()> {
    let tunnels = state
        .tunnels
        .lock()
        .map_err(|e| AppError::Ssh(format!("Lock error: {}", e)))?;

    tunnels.remove(&id);
    Ok(())
}
