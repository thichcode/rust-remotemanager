use crate::error::{AppError, AppResult};
use crate::ssh::tunnels::TunnelConfig;
use crate::AppState;
use tauri::State;
use uuid::Uuid;

/// List all tunnels for a given session (or all tunnels if session_id is empty).
#[tauri::command]
pub fn list_tunnels(state: State<AppState>, session_id: Option<String>) -> AppResult<Vec<TunnelConfig>> {
    let _sessions = state.sessions.lock();
    let tunnels = state.tunnels.lock();
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
    let sessions = state.sessions.lock();
    let ssh_session = sessions
        .get_ssh_session(&session_id)
        .ok_or_else(|| AppError::NotFound(format!("Session '{}' not found or not connected", session_id)))?;

    let tunnels = state.tunnels.lock();
    let mut config = config;
    config.id = Uuid::new_v4().to_string();
    config.session_id = session_id.clone();
    config.active = true;

    match config.tunnel_type {
        crate::ssh::tunnels::TunnelType::Local => tunnels.add_local(&ssh_session, config),
        crate::ssh::tunnels::TunnelType::Remote => tunnels.add_remote(&ssh_session, config),
        crate::ssh::tunnels::TunnelType::Dynamic => tunnels.add_dynamic(&ssh_session, config),
    }
}

/// Stop and remove a tunnel by ID.
#[tauri::command]
pub fn stop_tunnel(state: State<AppState>, id: String) -> AppResult<()> {
    let tunnels = state.tunnels.lock();
    tunnels.remove(&id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssh::tunnels::TunnelType;

    #[test]
    fn test_tunnel_config_serde() {
        let config = TunnelConfig {
            id: "abc-123".into(), session_id: "sess-1".into(),
            tunnel_type: TunnelType::Local, name: "test".into(),
            local_host: "127.0.0.1".into(), local_port: 8080,
            remote_host: "10.0.0.1".into(), remote_port: 80,
            active: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TunnelConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "abc-123");
        assert_eq!(deserialized.local_port, 8080);
        assert_eq!(deserialized.remote_port, 80);
    }

    #[test]
    fn test_tunnel_type_variants() {
        assert_eq!(format!("{:?}", TunnelType::Local), "Local");
        assert_eq!(format!("{:?}", TunnelType::Remote), "Remote");
        assert_eq!(format!("{:?}", TunnelType::Dynamic), "Dynamic");
    }

    #[test]
    fn test_tunnel_type_partial_eq() {
        assert_eq!(TunnelType::Local, TunnelType::Local);
        assert_ne!(TunnelType::Local, TunnelType::Remote);
        assert_ne!(TunnelType::Dynamic, TunnelType::Remote);
    }

    #[test]
    fn test_tunnel_config_default_active() {
        let config = TunnelConfig {
            id: "x".into(), session_id: "y".into(),
            tunnel_type: TunnelType::Local, name: "t".into(),
            local_host: "0.0.0.0".into(), local_port: 3000,
            remote_host: "example.com".into(), remote_port: 443,
            active: true,
        };
        assert!(config.active);
    }

    #[test]
    fn test_tunnel_config_inactive() {
        let config = TunnelConfig {
            id: "x".into(), session_id: "y".into(),
            tunnel_type: TunnelType::Remote, name: "t".into(),
            local_host: "127.0.0.1".into(), local_port: 3306,
            remote_host: "db.internal".into(), remote_port: 3306,
            active: false,
        };
        assert!(!config.active);
    }

    #[test]
    fn test_list_tunnels_filter_by_session() {
        // Test the filtering logic directly (no real tunnels needed)
        let configs = vec![
            TunnelConfig { id: "1".into(), session_id: "sess-a".into(), tunnel_type: TunnelType::Local, name: "t1".into(), local_host: "127.0.0.1".into(), local_port: 8080, remote_host: "r1".into(), remote_port: 80, active: true },
            TunnelConfig { id: "2".into(), session_id: "sess-b".into(), tunnel_type: TunnelType::Remote, name: "t2".into(), local_host: "127.0.0.1".into(), local_port: 9090, remote_host: "r2".into(), remote_port: 90, active: true },
            TunnelConfig { id: "3".into(), session_id: "sess-a".into(), tunnel_type: TunnelType::Dynamic, name: "t3".into(), local_host: "127.0.0.1".into(), local_port: 1080, remote_host: "r3".into(), remote_port: 1080, active: false },
        ];
        let filtered: Vec<&TunnelConfig> = configs.iter().filter(|t| t.session_id == "sess-a").collect();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, "1");
        assert_eq!(filtered[1].id, "3");
    }

    #[test]
    fn test_tunnel_config_clone() {
        let config = TunnelConfig {
            id: "clone-test".into(), session_id: "s".into(),
            tunnel_type: TunnelType::Local, name: "original".into(),
            local_host: "127.0.0.1".into(), local_port: 1234,
            remote_host: "remote".into(), remote_port: 5678,
            active: true,
        };
        let cloned = config.clone();
        assert_eq!(config.id, cloned.id);
        assert_eq!(config.name, cloned.name);
    }

    #[test]
    fn test_tunnel_config_display_fields() {
        let config = TunnelConfig {
            id: "display-test".into(), session_id: "s".into(),
            tunnel_type: TunnelType::Local, name: "web".into(),
            local_host: "127.0.0.1".into(), local_port: 8080,
            remote_host: "10.0.0.50".into(), remote_port: 80,
            active: true,
        };
        assert_eq!(config.local_host, "127.0.0.1");
        assert_eq!(config.local_port, 8080);
        assert_eq!(config.remote_host, "10.0.0.50");
        assert_eq!(config.remote_port, 80);
    }
}