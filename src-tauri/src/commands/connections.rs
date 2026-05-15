use crate::AppState;
use crate::error::AppResult;
use crate::logging;
use crate::storage::models::{Connection, ConnectionCreateRequest};
use crate::storage::repositories::connection_repo::ConnectionRepository;
use serde_json::json;
use tauri::State;

#[tauri::command]
pub fn list_connections(state: State<AppState>) -> AppResult<Vec<Connection>> {
    let db = state.db.lock().unwrap();
    let repo = ConnectionRepository::new(&db);
    Ok(repo.list()?)
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

    let db = state.db.lock().unwrap();
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
    conn: Connection,
) -> AppResult<()> {
    let db = state.db.lock().unwrap();
    let repo = ConnectionRepository::new(&db);
    repo.update(conn)?;
    Ok(())
}

#[tauri::command]
pub fn delete_connection(
    state: State<AppState>,
    id: String,
) -> AppResult<()> {
    let db = state.db.lock().unwrap();
    let repo = ConnectionRepository::new(&db);
    repo.delete(&id)?;
    Ok(())
}

#[tauri::command]
pub fn search_connections(
    state: State<AppState>,
    term: String,
) -> AppResult<Vec<Connection>> {
    let db = state.db.lock().unwrap();
    let repo = ConnectionRepository::new(&db);
    Ok(repo.search(&term)?)
}

#[tauri::command]
pub fn get_connection(
    state: State<AppState>,
    id: String,
) -> AppResult<Option<Connection>> {
    let db = state.db.lock().unwrap();
    let repo = ConnectionRepository::new(&db);
    Ok(repo.get_by_id(&id)?)
}

#[tauri::command]
pub fn get_favorites(state: State<AppState>) -> AppResult<Vec<Connection>> {
    let db = state.db.lock().unwrap();
    let repo = ConnectionRepository::new(&db);
    Ok(repo.get_favorites()?)
}
