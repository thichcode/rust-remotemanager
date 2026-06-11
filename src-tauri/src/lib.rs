//! Hermes Remote Manager — Rust backend
//!
//! This crate provides the core backend logic for the Hermes Remote Manager,
//! including the Tauri application setup, database initialization, state management,
//! IPC command handlers, and SSH session management.

#[cfg(test)]
mod test_support;

mod commands;
mod error;
mod logging;
mod security;
mod settings;
pub mod ssh;
mod storage;
mod ui;

use parking_lot::Mutex;
use rusqlite::Connection;
use security::vault::Vault;
use settings::SettingsManager;
use ssh::session::SessionManager;
use ssh::tunnels::TunnelManager;
use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;

/// Application state shared across all Tauri commands via `app.manage()`.
/// Uses `parking_lot::Mutex` for lock-free contention (no poisoning, faster).
pub struct AppState {
    pub db: Mutex<Connection>,
    pub vault: Mutex<Vault>,
    pub sessions: Mutex<SessionManager>,
    pub tunnels: Mutex<TunnelManager>,
    pub settings: Mutex<SettingsManager>,
    pub _logging_guard: WorkerGuard,
}

/// Initialize the database: create the app data directory, open the SQLite file,
/// run schema migrations via the storage module.
fn initialize_database(app: &tauri::App) -> Result<Connection, Box<dyn std::error::Error>> {
    use storage::database::Database;
    use storage::migrations;

    let app_dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir");
    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("hermes.db");
    let db = Database::new(db_path.to_str().unwrap())?;
    let conn = db.into_connection();
    migrations::run(&conn)?;

    tracing::info!("Database initialized at {:?}", db_path);
    Ok(conn)
}

/// Initialize the tracing / logging subscriber. Returns the guard that must
/// be kept alive for the duration of the program.
fn setup_logging() -> WorkerGuard {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "hermes_remote_manager=debug,tauri=warn".into());

    let log_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, "hermes.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_ansi(false)
        .compact()
        .init();

    tracing::info!("Logging initialized, writing to {:?}", log_dir);
    guard
}

// ──────────────────────────────────────────────
// Application entry point
// ──────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let logging_guard = setup_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .setup(move |app| {
            let conn = initialize_database(app).expect("failed to initialize database");
            let mut vault = Vault::new();
            let sessions = SessionManager::new();
            let tunnels = TunnelManager::new();
            let settings =
                SettingsManager::new(&conn).expect("failed to load settings");

            // Restore vault salt from persistent storage
            if let Some(salt_hex) = settings.get("vault_salt") {
                match hex::decode(salt_hex) {
                    Ok(salt_bytes) if salt_bytes.len() == 32 => {
                        let mut salt = [0u8; 32];
                        salt.copy_from_slice(&salt_bytes);
                        vault.set_salt(salt);
                        tracing::info!("Vault salt restored from storage");
                    }
                    Ok(_) => tracing::warn!(
                        "Vault salt has wrong length (expected 32 bytes), ignoring"
                    ),
                    Err(e) => tracing::warn!(
                        "Failed to decode vault salt from storage: {}",
                        e
                    ),
                }
            }

            let state = AppState {
                db: Mutex::new(conn),
                vault: Mutex::new(vault),
                sessions: Mutex::new(sessions),
                tunnels: Mutex::new(tunnels),
                settings: Mutex::new(settings),
                _logging_guard: logging_guard,
            };

            app.manage(state);

            // Launch Dioxus desktop UI
            tauri::async_runtime::spawn(async move {
                dioxus::launch(crate::ui::App);
            });

            tracing::info!("Hermes Remote Manager initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connections::list_connections,
            commands::connections::create_connection,
            commands::connections::update_connection,
            commands::connections::delete_connection,
            commands::connections::search_connections,
            commands::connections::get_connection,
            commands::connections::get_favorites,
            commands::folders::list_folders,
            commands::folders::create_folder,
            commands::folders::update_folder,
            commands::folders::delete_folder,
            commands::folders::reorder_folders,
            logging::log_debug,
            logging::log_json,
            logging::log_message,
            commands::credentials::pick_ssh_key_file,
            commands::credentials::list_credentials,
            commands::credentials::save_credential,
            commands::credentials::delete_credential,
            commands::terminal::connect_ssh,
            commands::terminal::disconnect_session,
            commands::terminal::terminal_input,
            commands::terminal::terminal_resize,
            commands::terminal::list_sessions,
            commands::terminal::get_session_state,
            commands::terminal::flush_session_output,
            commands::sftp::list_sftp_dir,
            commands::sftp::sftp_download,
            commands::sftp::sftp_upload,
            commands::sftp::sftp_mkdir,
            commands::sftp::sftp_rm,
            commands::sftp::sftp_rename,
            commands::sftp::sftp_stat,
            commands::tunnels::list_tunnels,
            commands::tunnels::create_tunnel,
            commands::tunnels::stop_tunnel,
            commands::vault::vault_status,
            commands::vault::vault_unlock,
            commands::vault::vault_lock,
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::rdp::connect_rdp,
            commands::app::get_app_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Hermes Remote Manager");
}