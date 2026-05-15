use crate::error::AppResult;
use crate::storage::models::Folder;
use chrono::Utc;
use rusqlite::{params, Connection as DbConnection};
use uuid::Uuid;

/// Repository for managing folder hierarchy in the connection tree.
pub struct FolderRepository<'a> {
    conn: &'a DbConnection,
}

impl<'a> FolderRepository<'a> {
    /// Create a new repository bound to the given database connection.
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }

    /// List all folders in tree order (sorted by sort_order, then name).
    pub fn list_all(&self) -> AppResult<Vec<Folder>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at
             FROM folders
             ORDER BY sort_order ASC, name ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get("id")?,
                name: row.get("name")?,
                parent_id: row.get("parent_id")?,
                sort_order: row.get("sort_order")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;

        let mut folders = Vec::new();
        for row in rows {
            folders.push(row?);
        }
        Ok(folders)
    }

    #[allow(dead_code)]
    /// List all folders sorted by sort_order (alias for list_all).
    pub fn list_tree(&self) -> AppResult<Vec<Folder>> {
        self.list_all()
    }

    #[allow(dead_code)]
    /// Get a single folder by ID.
    pub fn get_by_id(&self, id: &str) -> AppResult<Option<Folder>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at
             FROM folders WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Folder {
                id: row.get("id")?,
                name: row.get("name")?,
                parent_id: row.get("parent_id")?,
                sort_order: row.get("sort_order")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;

        match rows.next() {
            Some(Ok(folder)) => Ok(Some(folder)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Create a new folder with the given name, optional parent, and sort order.
    pub fn create(
        &self,
        name: &str,
        parent_id: Option<&str>,
        sort_order: i32,
    ) -> AppResult<Folder> {
        let now = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();

        self.conn.execute(
            "INSERT INTO folders (id, name, parent_id, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, parent_id, sort_order, now, now],
        )?;

        Ok(Folder {
            id,
            name: name.to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
            sort_order,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Update a folder's name and optionally its parent.
    /// Returns true if a row was modified.
    pub fn update(
        &self,
        id: &str,
        name: &str,
        parent_id: Option<&str>,
    ) -> AppResult<bool> {
        let now = Utc::now().to_rfc3339();
        let rows = self.conn.execute(
            "UPDATE folders SET name = ?1, parent_id = ?2, updated_at = ?3 WHERE id = ?4",
            params![name, parent_id, now, id],
        )?;
        Ok(rows > 0)
    }

    /// Delete a folder by ID. Children folders' parent_id will be set to NULL
    /// (ON DELETE SET NULL behavior, enforced via application logic).
    /// Returns true if a row was deleted.
    pub fn delete(&self, id: &str) -> AppResult<bool> {
        // Nullify parent_id in child folders
        self.conn.execute(
            "UPDATE folders SET parent_id = NULL WHERE parent_id = ?1",
            params![id],
        )?;

        // Nullify folder_id in connections
        self.conn.execute(
            "UPDATE connections SET folder_id = NULL WHERE folder_id = ?1",
            params![id],
        )?;

        let rows = self
            .conn
            .execute("DELETE FROM folders WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    /// Reorder folders by assigning new sort_order values based on the order
    /// of IDs in the provided vector.
    pub fn reorder(&self, ids: &[String]) -> AppResult<bool> {
        for (index, id) in ids.iter().enumerate() {
            self.conn.execute(
                "UPDATE folders SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
                params![index as i32, Utc::now().to_rfc3339(), id],
            )?;
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::Database;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_and_list() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = FolderRepository::new(db.get_conn());

        let folder = repo.create("SSH Servers", None, 0).unwrap();
        assert_eq!(folder.name, "SSH Servers");
        assert!(folder.parent_id.is_none());

        let folders = repo.list_all().unwrap();
        assert_eq!(folders.len(), 1);
    }

    #[test]
    fn test_nested_folders() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = FolderRepository::new(db.get_conn());

        let parent = repo.create("Production", None, 0).unwrap();
        let child = repo.create("Web Servers", Some(&parent.id), 0).unwrap();

        assert_eq!(child.parent_id, Some(parent.id.clone()));

        let folders = repo.list_tree().unwrap();
        assert_eq!(folders.len(), 2);
    }

    #[test]
    fn test_get_by_id() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = FolderRepository::new(db.get_conn());

        let created = repo.create("Test", None, 0).unwrap();
        let found = repo.get_by_id(&created.id).unwrap().unwrap();
        assert_eq!(found.name, "Test");

        let none = repo.get_by_id("nonexistent").unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn test_update() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = FolderRepository::new(db.get_conn());

        let folder = repo.create("Old Name", None, 0).unwrap();
        let updated = repo.update(&folder.id, "New Name", None).unwrap();
        assert!(updated);

        let found = repo.get_by_id(&folder.id).unwrap().unwrap();
        assert_eq!(found.name, "New Name");
    }

    #[test]
    fn test_delete_cascades_connections() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = FolderRepository::new(db.get_conn());
        let conn_repo = crate::storage::repositories::connection_repo::ConnectionRepository::new(db.get_conn());

        let folder = repo.create("To Delete", None, 0).unwrap();

        // Create a connection in this folder
        let req = crate::storage::models::ConnectionCreateRequest {
            name: "Test".into(),
            r#type: "ssh".into(),
            host: "10.0.0.1".into(),
            port: 22,
            username: "user".into(),
            auth_type: "password".into(),
            credential_id: None,
            password: None,
            private_key: None,
            folder_id: Some(folder.id.clone()),
            tags: None,
            notes: None,
            keepalive_interval: None,
            proxy_type: None,
            proxy_host: None,
            proxy_port: None,
            proxy_username: None,
            color: None,
        };
        conn_repo.create(req).unwrap();

        // Delete folder — connection should have folder_id set to NULL
        assert!(repo.delete(&folder.id).unwrap());

        let conns = conn_repo.list().unwrap();
        assert_eq!(conns.len(), 1);
        assert!(conns[0].folder_id.is_none());
    }

    #[test]
    fn test_reorder() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = FolderRepository::new(db.get_conn());

        let a = repo.create("A", None, 0).unwrap();
        let b = repo.create("B", None, 1).unwrap();

        repo.reorder(&[b.id.clone(), a.id.clone()]).unwrap();

        let folders = repo.list_all().unwrap();
        assert_eq!(folders[0].id, b.id);
        assert_eq!(folders[1].id, a.id);
    }
}
