use crate::AppState;
use crate::error::AppResult;
use crate::storage::models::{Connection, ConnectionCreateRequest};
use crate::storage::repositories::connection_repo::ConnectionRepository;
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
    let db = state.db.lock().unwrap();
    let repo = ConnectionRepository::new(&db);
    Ok(repo.create(req)?)
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
