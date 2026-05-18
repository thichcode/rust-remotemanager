use crate::AppState;
use crate::error::AppResult;
use crate::storage::models::Folder;
use crate::storage::repositories::folder_repo::FolderRepository;
use tauri::State;

#[tauri::command]
pub fn list_folders(state: State<AppState>) -> AppResult<Vec<Folder>> {
    let db = state.db.lock();
    let repo = FolderRepository::new(&db);
    repo.list_all()
}

#[tauri::command]
pub fn create_folder(
    state: State<AppState>,
    name: String,
    parent_id: Option<String>,
    sort_order: Option<i32>,
) -> AppResult<Folder> {
    let db = state.db.lock();
    let repo = FolderRepository::new(&db);
    let order = sort_order.unwrap_or(0);
    repo.create(&name, parent_id.as_deref(), order)
}

#[tauri::command]
pub fn update_folder(
    state: State<AppState>,
    id: String,
    name: String,
    parent_id: Option<String>,
) -> AppResult<()> {
    let db = state.db.lock();
    let repo = FolderRepository::new(&db);
    repo.update(&id, &name, parent_id.as_deref())?;
    Ok(())
}

#[tauri::command]
pub fn delete_folder(
    state: State<AppState>,
    id: String,
) -> AppResult<()> {
    let db = state.db.lock();
    let repo = FolderRepository::new(&db);
    repo.delete(&id)?;
    Ok(())
}

#[tauri::command]
pub fn reorder_folders(
    state: State<AppState>,
    ids: Vec<String>,
) -> AppResult<()> {
    let db = state.db.lock();
    let repo = FolderRepository::new(&db);
    repo.reorder(&ids)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;
    use crate::storage::migrations;
    use tempfile::NamedTempFile;

    fn setup_repo() -> (FolderRepository<'static>, Box<dyn std::any::Any>) {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let db = Database::new(&path).unwrap();
        let conn = db.into_connection();
        migrations::run(&conn).unwrap();
        let conn: &'static rusqlite::Connection = Box::leak(Box::new(conn));
        let repo = FolderRepository::new(conn);
        (repo, Box::new(tmp))
    }

    #[test]
    fn test_list_empty() {
        let (repo, _guard) = setup_repo();
        let folders = repo.list_all().unwrap();
        assert_eq!(folders.len(), 0);
    }

    #[test]
    fn test_create_root_folder() {
        let (repo, _guard) = setup_repo();
        let folder = repo.create("Production", None, 0).unwrap();
        assert_eq!(folder.name, "Production");
        assert!(folder.parent_id.is_none());
        assert_eq!(folder.sort_order, 0);
    }

    #[test]
    fn test_create_nested_folder() {
        let (repo, _guard) = setup_repo();
        let parent = repo.create("Parent", None, 0).unwrap();
        let child = repo.create("Child", Some(&parent.id), 1).unwrap();
        assert_eq!(child.parent_id, Some(parent.id.clone()));
        assert_eq!(child.sort_order, 1);
    }

    #[test]
    fn test_update_folder_name() {
        let (repo, _guard) = setup_repo();
        let folder = repo.create("Old", None, 0).unwrap();
        assert!(repo.update(&folder.id, "New", None).unwrap());
        let found = repo.get_by_id(&folder.id).unwrap().unwrap();
        assert_eq!(found.name, "New");
    }

    #[test]
    fn test_delete_folder() {
        let (repo, _guard) = setup_repo();
        let folder = repo.create("To Delete", None, 0).unwrap();
        assert!(repo.delete(&folder.id).unwrap());
        assert!(repo.get_by_id(&folder.id).unwrap().is_none());
    }

    #[test]
    fn test_reorder_folders() {
        let (repo, _guard) = setup_repo();
        let a = repo.create("A", None, 0).unwrap();
        let b = repo.create("B", None, 1).unwrap();
        repo.reorder(&[b.id.clone(), a.id.clone()]).unwrap();
        let folders = repo.list_all().unwrap();
        assert_eq!(folders[0].id, b.id);
        assert_eq!(folders[1].id, a.id);
    }

    #[test]
    fn test_delete_nonexistent() {
        let (repo, _guard) = setup_repo();
        let result = repo.delete("ghost-id").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_create_multiple_folders() {
        let (repo, _guard) = setup_repo();
        repo.create("Z", None, 0).unwrap();
        repo.create("A", None, 1).unwrap();
        repo.create("M", None, 2).unwrap();
        let folders = repo.list_all().unwrap();
        assert_eq!(folders.len(), 3);
        // Sorted by sort_order ASC, name ASC
        assert_eq!(folders[0].name, "Z");
        assert_eq!(folders[1].name, "A");
    }

    #[test]
    fn test_get_by_id_not_found() {
        let (repo, _guard) = setup_repo();
        let result = repo.get_by_id("nonexistent").unwrap();
        assert!(result.is_none());
    }
}