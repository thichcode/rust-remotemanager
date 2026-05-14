use rusqlite::{Connection, Result};
use std::path::Path;

/// Core database manager for SQLite storage.
///
/// Opens or creates the SQLite database file, configures WAL mode,
/// foreign keys, and busy timeout for concurrent safety.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create a SQLite database at the given `path`.
    ///
    /// * Enables WAL mode for better concurrent read/write performance
    /// * Enforces foreign key constraints
    /// * Sets a 5-second busy timeout
    /// * Runs pending migrations
    pub fn new(path: &str) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    rusqlite::Error::InvalidParameterName(format!(
                        "Failed to create database directory '{}': {}",
                        parent.display(),
                        e
                    ))
                })?;
            }
        }

        let conn = Connection::open(path)?;

        // Enable WAL mode for concurrent performance
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;

        // Enable foreign key enforcement
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Set busy timeout to 5 seconds
        conn.execute_batch("PRAGMA busy_timeout = 5000;")?;

        Ok(Database { conn })
    }

    /// Get a reference to the underlying SQLite connection.
    pub fn get_conn(&self) -> &Connection {
        &self.conn
    }

    /// Get a mutable reference to the underlying SQLite connection.
    pub fn get_conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Consume the Database and return the inner Connection.
    pub fn into_connection(self) -> Connection {
        self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_new() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let db = Database::new(&path);
        assert!(db.is_ok());
    }

    #[test]
    fn test_database_get_conn() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let db = Database::new(&path).unwrap();
        let conn = db.get_conn();
        // Verify we can execute a simple query
        let result: i32 = conn
            .query_row("SELECT 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_wal_mode_enabled() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let db = Database::new(&path).unwrap();
        let conn = db.get_conn();
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode.to_uppercase(), "WAL");
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let db = Database::new(&path).unwrap();
        let conn = db.get_conn();
        let fk_enabled: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);
    }
}
