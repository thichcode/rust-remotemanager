use crate::error::AppResult;
use crate::storage::models::{Connection, ConnectionCreateRequest};
use chrono::Utc;
use rusqlite::{params, Connection as DbConnection};
use uuid::Uuid;

/// Repository for managing connection profiles in the database.
pub struct ConnectionRepository<'a> {
    conn: &'a DbConnection,
}

impl<'a> ConnectionRepository<'a> {
    /// Create a new repository bound to the given database connection.
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }

    /// List all connections, ordered by sort_order and name.
    pub fn list(&self) -> AppResult<Vec<Connection>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, type, folder_id, host, port, username,
                    credential_id, auth_type,
                    proxy_type, proxy_host, proxy_port, proxy_username,
                    tags, notes, startup_commands, keepalive_interval,
                    is_favorite, color, sort_order, created_at, updated_at
             FROM connections
             ORDER BY sort_order ASC, name ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Connection {
                id: row.get("id")?,
                name: row.get("name")?,
                r#type: row.get("type")?,
                folder_id: row.get("folder_id")?,
                host: row.get("host")?,
                port: row.get("port")?,
                username: row.get("username")?,
                credential_id: row.get("credential_id")?,
                auth_type: row.get("auth_type")?,
                proxy_type: row.get("proxy_type")?,
                proxy_host: row.get("proxy_host")?,
                proxy_port: row.get("proxy_port")?,
                proxy_username: row.get("proxy_username")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                startup_commands: row.get("startup_commands")?,
                keepalive_interval: row.get("keepalive_interval")?,
                is_favorite: row.get::<_, i32>("is_favorite")? != 0,
                color: row.get("color")?,
                sort_order: row.get("sort_order")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;

        let mut connections = Vec::new();
        for row in rows {
            connections.push(row?);
        }
        Ok(connections)
    }

    /// Get a single connection by its ID.
    pub fn get_by_id(&self, id: &str) -> AppResult<Option<Connection>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, type, folder_id, host, port, username,
                    credential_id, auth_type,
                    proxy_type, proxy_host, proxy_port, proxy_username,
                    tags, notes, startup_commands, keepalive_interval,
                    is_favorite, color, sort_order, created_at, updated_at
             FROM connections WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Connection {
                id: row.get("id")?,
                name: row.get("name")?,
                r#type: row.get("type")?,
                folder_id: row.get("folder_id")?,
                host: row.get("host")?,
                port: row.get("port")?,
                username: row.get("username")?,
                credential_id: row.get("credential_id")?,
                auth_type: row.get("auth_type")?,
                proxy_type: row.get("proxy_type")?,
                proxy_host: row.get("proxy_host")?,
                proxy_port: row.get("proxy_port")?,
                proxy_username: row.get("proxy_username")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                startup_commands: row.get("startup_commands")?,
                keepalive_interval: row.get("keepalive_interval")?,
                is_favorite: row.get::<_, i32>("is_favorite")? != 0,
                color: row.get("color")?,
                sort_order: row.get("sort_order")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;

        match rows.next() {
            Some(Ok(conn)) => Ok(Some(conn)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Create a new connection from a request, generating UUID and timestamps.
    pub fn create(&self, req: ConnectionCreateRequest) -> AppResult<Connection> {
        let now = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();
        let is_favorite = false;
        let sort_order = 0;
        let keepalive_interval = req.keepalive_interval.or(Some(30));

        self.conn.execute(
            "INSERT INTO connections (
                id, name, type, folder_id, host, port, username,
                credential_id, auth_type,
                proxy_type, proxy_host, proxy_port, proxy_username,
                tags, notes, startup_commands, keepalive_interval,
                is_favorite, color, sort_order, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                      ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17,
                      ?18, ?19, ?20, ?21, ?22)",
            params![
                id,
                req.name,
                req.r#type,
                req.folder_id,
                req.host,
                req.port,
                req.username,
                req.credential_id,
                req.auth_type,
                req.proxy_type,
                req.proxy_host,
                req.proxy_port,
                req.proxy_username,
                req.tags,
                req.notes,
                None::<String>, // startup_commands not in create request
                keepalive_interval,
                is_favorite as i32,
                req.color,
                sort_order,
                now,
                now,
            ],
        )?;

        Ok(Connection {
            id,
            name: req.name,
            r#type: req.r#type,
            folder_id: req.folder_id,
            host: req.host,
            port: req.port,
            username: req.username,
            credential_id: req.credential_id,
            auth_type: req.auth_type,
            proxy_type: req.proxy_type,
            proxy_host: req.proxy_host,
            proxy_port: req.proxy_port,
            proxy_username: req.proxy_username,
            tags: req.tags,
            notes: req.notes,
            startup_commands: None, // Not in request; set to None
            keepalive_interval,
            is_favorite,
            color: req.color,
            sort_order,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Update an existing connection. Returns true if any row was modified.
    pub fn update(&self, conn: Connection) -> AppResult<bool> {
        let now = Utc::now().to_rfc3339();
        let rows = self.conn.execute(
            "UPDATE connections SET
                name = ?1, type = ?2, folder_id = ?3, host = ?4,
                port = ?5, username = ?6, credential_id = ?7,
                auth_type = ?8,
                proxy_type = ?9, proxy_host = ?10, proxy_port = ?11,
                proxy_username = ?12, tags = ?13, notes = ?14,
                startup_commands = ?15, keepalive_interval = ?16,
                is_favorite = ?17, color = ?18, sort_order = ?19,
                updated_at = ?20
             WHERE id = ?21",
            params![
                conn.name,
                conn.r#type,
                conn.folder_id,
                conn.host,
                conn.port,
                conn.username,
                conn.credential_id,
                conn.auth_type,
                conn.proxy_type,
                conn.proxy_host,
                conn.proxy_port,
                conn.proxy_username,
                conn.tags,
                conn.notes,
                conn.startup_commands,
                conn.keepalive_interval,
                conn.is_favorite as i32,
                conn.color,
                conn.sort_order,
                now,
                conn.id,
            ],
        )?;
        Ok(rows > 0)
    }

    /// Delete a connection by ID. Returns true if a row was deleted.
    pub fn delete(&self, id: &str) -> AppResult<bool> {
        let rows = self
            .conn
            .execute("DELETE FROM connections WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    /// Search connections by name, host, or username (case-insensitive LIKE).
    pub fn search(&self, term: &str) -> AppResult<Vec<Connection>> {
        let pattern = format!("%{}%", term);
        let mut stmt = self.conn.prepare(
            "SELECT id, name, type, folder_id, host, port, username,
                    credential_id, auth_type,
                    proxy_type, proxy_host, proxy_port, proxy_username,
                    tags, notes, startup_commands, keepalive_interval,
                    is_favorite, color, sort_order, created_at, updated_at
             FROM connections
             WHERE name LIKE ?1 OR host LIKE ?1 OR username LIKE ?1
             ORDER BY sort_order ASC, name ASC",
        )?;

        let rows = stmt.query_map(params![pattern], |row| {
            Ok(Connection {
                id: row.get("id")?,
                name: row.get("name")?,
                r#type: row.get("type")?,
                folder_id: row.get("folder_id")?,
                host: row.get("host")?,
                port: row.get("port")?,
                username: row.get("username")?,
                credential_id: row.get("credential_id")?,
                auth_type: row.get("auth_type")?,
                proxy_type: row.get("proxy_type")?,
                proxy_host: row.get("proxy_host")?,
                proxy_port: row.get("proxy_port")?,
                proxy_username: row.get("proxy_username")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                startup_commands: row.get("startup_commands")?,
                keepalive_interval: row.get("keepalive_interval")?,
                is_favorite: row.get::<_, i32>("is_favorite")? != 0,
                color: row.get("color")?,
                sort_order: row.get("sort_order")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;

        let mut connections = Vec::new();
        for row in rows {
            connections.push(row?);
        }
        Ok(connections)
    }

    /// Get all connections within a specific folder.
    pub fn get_by_folder(&self, folder_id: &str) -> AppResult<Vec<Connection>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, type, folder_id, host, port, username,
                    credential_id, auth_type,
                    proxy_type, proxy_host, proxy_port, proxy_username,
                    tags, notes, startup_commands, keepalive_interval,
                    is_favorite, color, sort_order, created_at, updated_at
             FROM connections WHERE folder_id = ?1
             ORDER BY sort_order ASC, name ASC",
        )?;

        let rows = stmt.query_map(params![folder_id], |row| {
            Ok(Connection {
                id: row.get("id")?,
                name: row.get("name")?,
                r#type: row.get("type")?,
                folder_id: row.get("folder_id")?,
                host: row.get("host")?,
                port: row.get("port")?,
                username: row.get("username")?,
                credential_id: row.get("credential_id")?,
                auth_type: row.get("auth_type")?,
                proxy_type: row.get("proxy_type")?,
                proxy_host: row.get("proxy_host")?,
                proxy_port: row.get("proxy_port")?,
                proxy_username: row.get("proxy_username")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                startup_commands: row.get("startup_commands")?,
                keepalive_interval: row.get("keepalive_interval")?,
                is_favorite: row.get::<_, i32>("is_favorite")? != 0,
                color: row.get("color")?,
                sort_order: row.get("sort_order")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;

        let mut connections = Vec::new();
        for row in rows {
            connections.push(row?);
        }
        Ok(connections)
    }

    /// Get all favorite connections.
    pub fn get_favorites(&self) -> AppResult<Vec<Connection>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, type, folder_id, host, port, username,
                    credential_id, auth_type,
                    proxy_type, proxy_host, proxy_port, proxy_username,
                    tags, notes, startup_commands, keepalive_interval,
                    is_favorite, color, sort_order, created_at, updated_at
             FROM connections WHERE is_favorite = 1
             ORDER BY sort_order ASC, name ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Connection {
                id: row.get("id")?,
                name: row.get("name")?,
                r#type: row.get("type")?,
                folder_id: row.get("folder_id")?,
                host: row.get("host")?,
                port: row.get("port")?,
                username: row.get("username")?,
                credential_id: row.get("credential_id")?,
                auth_type: row.get("auth_type")?,
                proxy_type: row.get("proxy_type")?,
                proxy_host: row.get("proxy_host")?,
                proxy_port: row.get("proxy_port")?,
                proxy_username: row.get("proxy_username")?,
                tags: row.get("tags")?,
                notes: row.get("notes")?,
                startup_commands: row.get("startup_commands")?,
                keepalive_interval: row.get("keepalive_interval")?,
                is_favorite: row.get::<_, i32>("is_favorite")? != 0,
                color: row.get("color")?,
                sort_order: row.get("sort_order")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;

        let mut connections = Vec::new();
        for row in rows {
            connections.push(row?);
        }
        Ok(connections)
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
        let repo = ConnectionRepository::new(db.get_conn());

        let req = ConnectionCreateRequest {
            name: "Test Server".into(),
            r#type: "ssh".into(),
            host: "192.168.1.1".into(),
            port: 22,
            username: "admin".into(),
            auth_type: "password".into(),
            credential_id: None,
            password: None,
            private_key: None,
            folder_id: None,
            tags: None,
            notes: None,
            keepalive_interval: None,
            proxy_type: None,
            proxy_host: None,
            proxy_port: None,
            proxy_username: None,
            color: None,
        };

        let conn = repo.create(req).unwrap();
        assert_eq!(conn.name, "Test Server");
        assert_eq!(conn.host, "192.168.1.1");

        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_get_by_id_not_found() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = ConnectionRepository::new(db.get_conn());

        let result = repo.get_by_id("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_delete() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = ConnectionRepository::new(db.get_conn());

        let req = ConnectionCreateRequest {
            name: "To Delete".into(),
            r#type: "ssh".into(),
            host: "10.0.0.1".into(),
            port: 22,
            username: "user".into(),
            auth_type: "password".into(),
            credential_id: None,
            password: None,
            private_key: None,
            folder_id: None,
            tags: None,
            notes: None,
            keepalive_interval: None,
            proxy_type: None,
            proxy_host: None,
            proxy_port: None,
            proxy_username: None,
            color: None,
        };

        let c = repo.create(req).unwrap();
        assert!(repo.delete(&c.id).unwrap());
        assert!(repo.get_by_id(&c.id).unwrap().is_none());
    }

    #[test]
    fn test_search() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = ConnectionRepository::new(db.get_conn());

        let req1 = ConnectionCreateRequest {
            name: "Production DB".into(),
            r#type: "ssh".into(),
            host: "db.example.com".into(),
            port: 22,
            username: "root".into(),
            auth_type: "password".into(),
            credential_id: None,
            password: None,
            private_key: None,
            folder_id: None,
            tags: None,
            notes: None,
            keepalive_interval: None,
            proxy_type: None,
            proxy_host: None,
            proxy_port: None,
            proxy_username: None,
            color: None,
        };
        repo.create(req1).unwrap();

        let results = repo.search("Production").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Production DB");

        let no_results = repo.search("Nonexistent").unwrap();
        assert_eq!(no_results.len(), 0);
    }
}
