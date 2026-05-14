use crate::AppState;
use crate::error::AppResult;
use crate::storage::models::Folder;
use crate::storage::repositories::folder_repo::FolderRepository;
use tauri::State;

#[tauri::command]
pub fn list_folders(state: State<AppState>) -> AppResult<Vec<Folder>> {
    let db = state.db.lock().unwrap();
    let repo = FolderRepository::new(&db);
    Ok(repo.list_all()?)
}

#[tauri::command]
pub fn create_folder(
    state: State<AppState>,
    name: String,
    parent_id: Option<String>,
    sort_order: Option<i32>,
) -> AppResult<Folder> {
    let db = state.db.lock().unwrap();
    let repo = FolderRepository::new(&db);
    let order = sort_order.unwrap_or(0);
    Ok(repo.create(&name, parent_id.as_deref(), order)?)
}

#[tauri::command]
pub fn update_folder(
    state: State<AppState>,
    id: String,
    name: String,
    parent_id: Option<String>,
) -> AppResult<()> {
    let db = state.db.lock().unwrap();
    let repo = FolderRepository::new(&db);
    repo.update(&id, &name, parent_id.as_deref())?;
    Ok(())
}

#[tauri::command]
pub fn delete_folder(
    state: State<AppState>,
    id: String,
) -> AppResult<()> {
    let db = state.db.lock().unwrap();
    let repo = FolderRepository::new(&db);
    repo.delete(&id)?;
    Ok(())
}

#[tauri::command]
pub fn reorder_folders(
    state: State<AppState>,
    ids: Vec<String>,
) -> AppResult<()> {
    let db = state.db.lock().unwrap();
    let repo = FolderRepository::new(&db);
    repo.reorder(&ids)?;
    Ok(())
}
