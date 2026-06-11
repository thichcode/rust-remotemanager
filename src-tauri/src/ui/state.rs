use std::sync::Arc;
use parking_lot::Mutex;
use crate::storage::database::Database;
use crate::ssh::session::SessionManager;
use crate::ssh::tunnels::TunnelManager;
use crate::security::vault::Vault;
use crate::settings::SettingsManager;

/// Persistent backend state for the application.
///
/// Reactive UI state (connections, folders, active session, theme, etc.)
/// should be managed with `use_signal` at the component level, not here.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub vault: Arc<Mutex<Vault>>,
    pub sessions: Arc<Mutex<SessionManager>>,
    pub tunnels: Arc<Mutex<TunnelManager>>,
    pub settings: Arc<Mutex<SettingsManager>>,
    pub sidebar_collapsed: Arc<Mutex<bool>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl AppState {
    pub fn new(
        db: Arc<Mutex<Database>>,
        vault: Arc<Mutex<Vault>>,
        sessions: Arc<Mutex<SessionManager>>,
        tunnels: Arc<Mutex<TunnelManager>>,
        settings: Arc<Mutex<SettingsManager>>,
    ) -> Self {
        Self {
            db,
            vault,
            sessions,
            tunnels,
            settings,
            sidebar_collapsed: Arc::new(Mutex::new(false)),
        }
    }
}
