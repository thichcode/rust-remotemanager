use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};

/// Configuration for establishing an SSH connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub key_content: Option<String>,
    pub passphrase: Option<String>,
    pub keepalive_interval: Option<u32>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<u16>,
    pub proxy_username: Option<String>,
}

/// State of an SSH session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Represents an active SSH terminal session.
#[derive(Debug, Clone)]
pub struct SshSession {
    pub id: String,
    pub config: SshConfig,
    pub state: SessionState,
}

/// Manages all active SSH sessions.
pub struct SessionManager {
    sessions: Mutex<HashMap<String, SshSession>>,
}

impl SessionManager {
    /// Create a new empty session manager.
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new session with the given ID and config.
    pub fn create(&self, id: String, config: SshConfig) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(
            id.clone(),
            SshSession {
                id,
                config,
                state: SessionState::Connecting,
            },
        );
    }

    /// Get a session by ID (returns None if not found).
    pub fn get(&self, id: &str) -> Option<SshSession> {
        self.sessions.lock().unwrap().get(id).cloned()
    }

    /// Remove a session by ID.
    pub fn remove(&self, id: &str) {
        self.sessions.lock().unwrap().remove(id);
    }

    /// List all active session IDs.
    pub fn list(&self) -> Vec<String> {
        self.sessions.lock().unwrap().keys().cloned().collect()
    }

    /// Update the state of a session.
    pub fn update_state(&self, id: &str, state: SessionState) {
        if let Some(session) = self.sessions.lock().unwrap().get_mut(id) {
            session.state = state;
        }
    }

    /// Return the number of active sessions.
    pub fn active_count(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> SshConfig {
        SshConfig {
            host: "192.168.1.1".into(),
            port: 22,
            username: "admin".into(),
            auth_type: "password".into(),
            password: Some("secret".into()),
            key_path: None,
            key_content: None,
            passphrase: None,
            keepalive_interval: Some(30),
            proxy_host: None,
            proxy_port: None,
            proxy_username: None,
        }
    }

    #[test]
    fn test_create_and_get() {
        let mgr = SessionManager::new();
        let config = make_config();
        mgr.create("session-1".into(), config.clone());
        let session = mgr.get("session-1").unwrap();
        assert_eq!(session.id, "session-1");
        assert_eq!(session.config.host, "192.168.1.1");
        assert_eq!(session.state, SessionState::Connecting);
    }

    #[test]
    fn test_get_nonexistent() {
        let mgr = SessionManager::new();
        assert!(mgr.get("ghost").is_none());
    }

    #[test]
    fn test_remove() {
        let mgr = SessionManager::new();
        mgr.create("s1".into(), make_config());
        mgr.remove("s1");
        assert!(mgr.get("s1").is_none());
    }

    #[test]
    fn test_list() {
        let mgr = SessionManager::new();
        mgr.create("a".into(), make_config());
        mgr.create("b".into(), make_config());
        let list = mgr.list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"a".to_string()));
        assert!(list.contains(&"b".to_string()));
    }

    #[test]
    fn test_update_state() {
        let mgr = SessionManager::new();
        mgr.create("s1".into(), make_config());
        mgr.update_state("s1", SessionState::Connected);
        let session = mgr.get("s1").unwrap();
        assert_eq!(session.state, SessionState::Connected);
    }

    #[test]
    fn test_active_count() {
        let mgr = SessionManager::new();
        assert_eq!(mgr.active_count(), 0);
        mgr.create("s1".into(), make_config());
        assert_eq!(mgr.active_count(), 1);
        mgr.remove("s1");
        assert_eq!(mgr.active_count(), 0);
    }
}
