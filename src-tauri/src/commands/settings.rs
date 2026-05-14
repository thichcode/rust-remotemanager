use crate::AppState;
use crate::error::AppResult;
use tauri::State;
use std::collections::HashMap;

/// Get all application settings as a key-value map.
#[tauri::command]
pub fn get_settings(state: State<AppState>) -> HashMap<String, String> {
    let settings = state.settings.lock().unwrap();
    settings.get_all()
}

/// Update a single application setting.
#[tauri::command]
pub fn update_setting(
    state: State<AppState>,
    key: String,
    value: String,
) -> AppResult<()> {
    let db = state.db.lock().unwrap();
    let mut settings = state.settings.lock().unwrap();
    settings.set(&db, &key, &value)?;
    Ok(())
}
