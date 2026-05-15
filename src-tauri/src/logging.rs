use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

/// Append a timestamped message to the app's log.txt file.
#[tauri::command]
pub fn log_message(message: String) -> Result<(), String> {
    let log_path = std::path::PathBuf::from("log.txt");
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let entry = format!("[{}] {}\n", timestamp, message);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    file.write_all(entry.as_bytes())
        .map_err(|e| format!("Failed to write log: {}", e))?;

    Ok(())
}

/// Log a structured JSON entry for debugging IPC flows.
#[tauri::command]
pub fn log_debug(tag: String, payload: String) -> Result<(), String> {
    let log_path = std::path::PathBuf::from("log.txt");
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let entry = format!("[{}] [{}] {}\n", timestamp, tag, payload);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    file.write_all(entry.as_bytes())
        .map_err(|e| format!("Failed to write log: {}", e))?;

    Ok(())
}

/// Convenience: log a tag + JSON-serializable payload in one call.
#[tauri::command]
pub fn log_json(tag: String, payload: serde_json::Value) -> Result<(), String> {
    log_debug(tag, serde_json::to_string(&payload).unwrap_or_default())
}