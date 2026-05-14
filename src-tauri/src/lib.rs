//! Hermes Remote Manager — Rust backend
//!
//! This crate provides the core backend logic for the Hermes Remote Manager,
//! including the Tauri application setup, database initialization, state management,
//! IPC command handlers, and SSH session management.

mod commands;
mod error;
mod security;
mod settings;
pub mod ssh;
mod storage;

use rusqlite::Connection;
use security::vault::Vault;
use ssh::session::SessionManager;
use settings::SettingsManager;
use std::sync::Mutex;
use tauri::Manager;

/// Application state shared across all Tauri commands via `app.manage()`.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub vault: Mutex<Vault>,
    pub sessions: Mutex<SessionManager>,
    pub settings: Mutex<SettingsManager>,
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

/// Initialize the tracing / logging subscriber with sensible defaults.
fn setup_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hermes_remote_manager=info,tauri=info".into()),
        )
        .compact()
        .init();
}

// ──────────────────────────────────────────────
// Application entry point
// ──────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let conn = initialize_database(app).expect("failed to initialize database");
            let vault = Vault::new();
            let sessions = SessionManager::new();
            let settings =
                SettingsManager::new(&conn).expect("failed to load settings");

            let state = AppState {
                db: Mutex::new(conn),
                vault: Mutex::new(vault),
                sessions: Mutex::new(sessions),
                settings: Mutex::new(settings),
            };

            app.manage(state);

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
            commands::credentials::list_credentials,
            commands::credentials::save_credential,
            commands::credentials::delete_credential,
            commands::terminal::connect_ssh,
            commands::terminal::disconnect_session,
            commands::terminal::terminal_input,
            commands::terminal::terminal_resize,
            commands::terminal::list_sessions,
            commands::vault::vault_status,
            commands::vault::vault_unlock,
            commands::vault::vault_lock,
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::app::get_app_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Hermes Remote Manager");
}
