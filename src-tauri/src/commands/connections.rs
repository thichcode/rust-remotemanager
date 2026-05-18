use crate::AppState;
use crate::error::AppResult;
use crate::logging;
use crate::storage::models::{Connection, ConnectionCreateRequest};
use crate::storage::repositories::connection_repo::ConnectionRepository;
use serde_json::json;
use tauri::State;

#[tauri::command]
pub fn list_connections(state: State<AppState>) -> AppResult<Vec<Connection>> {
    let db = state.db.lock();
    let repo = ConnectionRepository::new(&db);
    repo.list()
}

#[tauri::command]
pub fn create_connection(
    state: State<AppState>,
    req: ConnectionCreateRequest,
) -> AppResult<Connection> {
    let payload = json!({
        "name": req.name,
        "type": req.r#type,
        "host": req.host,
        "port": req.port,
        "username": req.username,
        "auth_type": req.auth_type,
        "credential_id": req.credential_id,
        "folder_id": req.folder_id,
        "proxy_type": req.proxy_type,
    });
    tracing::info!("create_connection called: {}", payload.to_string());
    logging::log_debug("create_connection".into(), payload.to_string())
        .map_err(|e| format!("log write failed: {}", e))?;

    // Log the raw incoming request for debugging
    let req_json = serde_json::to_string(&req).unwrap_or_default();
    logging::log_debug("create_connection_raw_req".into(), req_json)
        .map_err(|e| format!("log write failed: {}", e))?;

    let db = state.db.lock();
    let repo = ConnectionRepository::new(&db);
    let result = repo.create(req);
    match &result {
        Ok(conn) => {
            tracing::info!("create_connection success: id={}", conn.id);
            logging::log_debug("create_connection_success".into(), json!({"id": &conn.id, "name": &conn.name}).to_string())
                .map_err(|e| format!("log write failed: {}", e))?;
        }
        Err(e) => {
            tracing::error!("create_connection failed: {}", e);
            logging::log_debug("create_connection_error".into(), e.to_string())
                .map_err(|e| format!("log write failed: {}", e))?;
        }
    }
    result
}

#[tauri::command]
pub fn update_connection(
    state: State<AppState>,
    conn: serde_json::Value,
) -> AppResult<()> {
    tracing::info!("[update_connection] raw payload: {}", conn);
    let conn_raw = conn.clone();
    let conn: Connection = serde_json::from_value(conn).map_err(|e| {
        crate::error::AppError::Validation(format!("parse update payload: {} — input: {}", e, conn_raw))
    })?;
    tracing::info!(
        "[update_connection] parsed: id={}, credential_id={:?}, is_favorite={}",
        conn.id, conn.credential_id, conn.is_favorite
    );
    let db = state.db.lock();
    let repo = ConnectionRepository::new(&db);
    repo.update(conn)?;
    Ok(())
}

#[tauri::command]
pub fn delete_connection(
    state: State<AppState>,
    id: String,
) -> AppResult<()> {
    let db = state.db.lock();
    let repo = ConnectionRepository::new(&db);
    repo.delete(&id)?;
    Ok(())
}

#[tauri::command]
pub fn search_connections(
    state: State<AppState>,
    term: String,
) -> AppResult<Vec<Connection>> {
    let db = state.db.lock();
    let repo = ConnectionRepository::new(&db);
    repo.search(&term)
}

#[tauri::command]
pub fn get_connection(
    state: State<AppState>,
    id: String,
) -> AppResult<Option<Connection>> {
    let db = state.db.lock();
    let repo = ConnectionRepository::new(&db);
    repo.get_by_id(&id)
}

#[tauri::command]
pub fn get_favorites(state: State<AppState>) -> AppResult<Vec<Connection>> {
    let db = state.db.lock();
    let repo = ConnectionRepository::new(&db);
    repo.get_favorites()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use crate::storage::database::Database;
    use crate::storage::migrations;
    use tempfile::NamedTempFile;

    fn setup_repo() -> (ConnectionRepository<'static>, Box<dyn std::any::Any>) {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let db = Database::new(&path).unwrap();
        let conn = db.into_connection();
        migrations::run(&conn).unwrap();
        // Leak to get 'static lifetime for testing
        let conn: &'static rusqlite::Connection = Box::leak(Box::new(conn));
        let repo = ConnectionRepository::new(conn);
        (repo, Box::new(tmp))
    }

    #[test]
    fn test_list_empty() {
        let (repo, _guard) = setup_repo();
        let list = repo.list().unwrap();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_create_and_get() {
        let (repo, _guard) = setup_repo();
        let req = ConnectionCreateRequest {
            name: "Test Server".into(), r#type: "ssh".into(),
            host: "192.168.1.1".into(), port: 22, username: "admin".into(),
            auth_type: "password".into(), credential_id: None,
            password: None, private_key: None, folder_id: None,
            tags: None, notes: None, keepalive_interval: None,
            proxy_type: None, proxy_host: None, proxy_port: None,
            proxy_username: None, color: None,
        };
        let conn = repo.create(req).unwrap();
        assert_eq!(conn.name, "Test Server");
        let found = repo.get_by_id(&conn.id).unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn test_get_favorites_empty() {
        let (repo, _guard) = setup_repo();
        let favs = repo.get_favorites().unwrap();
        assert_eq!(favs.len(), 0);
    }

    #[test]
    fn test_update_toggle_favorite() {
        let (repo, _guard) = setup_repo();
        let req = ConnectionCreateRequest {
            name: "Fav".into(), r#type: "ssh".into(),
            host: "10.0.0.1".into(), port: 22, username: "u".into(),
            auth_type: "password".into(), credential_id: None,
            password: None, private_key: None, folder_id: None,
            tags: None, notes: None, keepalive_interval: None,
            proxy_type: None, proxy_host: None, proxy_port: None,
            proxy_username: None, color: None,
        };
        let mut c = repo.create(req).unwrap();
        assert!(!c.is_favorite);
        c.is_favorite = true;
        repo.update(c.clone()).unwrap();
        let updated = repo.get_by_id(&c.id).unwrap().unwrap();
        assert!(updated.is_favorite);
    }

    #[test]
    fn test_delete_nonexistent_returns_false() {
        let (repo, _guard) = setup_repo();
        let result = repo.delete("nonexistent-id").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_validation_error() {
        let err = AppError::Validation("bad data".into());
        assert!(err.to_string().contains("bad data"));
    }

    #[test]
    fn test_search_no_results() {
        let (repo, _guard) = setup_repo();
        let results = repo.search("zzzz_nonexistent").unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_create_multiple_and_count() {
        let (repo, _guard) = setup_repo();
        for i in 0..5 {
            let req = ConnectionCreateRequest {
                name: format!("Server {}", i), r#type: "ssh".into(),
                host: format!("10.0.0.{}", i + 1).into(), port: 22,
                username: "admin".into(), auth_type: "password".into(),
                credential_id: None, password: None, private_key: None,
                folder_id: None, tags: None, notes: None,
                keepalive_interval: None, proxy_type: None,
                proxy_host: None, proxy_port: None,
                proxy_username: None, color: None,
            };
            repo.create(req).unwrap();
        }
        assert_eq!(repo.list().unwrap().len(), 5);
    }
}