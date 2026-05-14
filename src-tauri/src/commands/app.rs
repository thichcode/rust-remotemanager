use serde_json::Value;

/// Get application metadata (name, version, description).
#[tauri::command]
pub fn get_app_info() -> Value {
    serde_json::json!({
        "name": "Hermes Remote Manager",
        "version": "0.1.0",
        "description": "Remote system management tool"
    })
}
