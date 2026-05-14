use crate::AppState;
use crate::error::{AppError, AppResult};
use tauri::State;

/// Check whether the vault is currently unlocked.
#[tauri::command]
pub fn vault_status(state: State<AppState>) -> bool {
    let vault = state.vault.lock().unwrap();
    vault.is_unlocked()
}

/// Unlock the vault with the given master password.
#[tauri::command]
pub fn vault_unlock(
    state: State<AppState>,
    password: String,
) -> AppResult<()> {
    let mut vault = state.vault.lock().unwrap();
    vault.unlock(&password).map_err(|e| {
        AppError::Vault(format!("Failed to unlock vault: {}", e))
    })
}

/// Lock the vault, zeroizing the master key in memory.
#[tauri::command]
pub fn vault_lock(state: State<AppState>) -> AppResult<()> {
    let mut vault = state.vault.lock().unwrap();
    vault.lock();
    Ok(())
}
