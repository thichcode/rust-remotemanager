use rusqlite::{params, Connection};
use std::sync::Mutex;

/// Result type for keychain operations.
pub type Result<T> = std::result::Result<T, KeychainError>;

/// Errors that can occur during keychain operations.
#[derive(Debug, thiserror::Error)]
pub enum KeychainError {
    #[error("Keychain backend error: {0}")]
    BackendError(String),
    #[error("Entry not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
}

/// Trait defining the interface for secure credential storage backends.
///
/// Implementations can wrap platform-specific secure storage:
/// - Windows: Credential Manager (WinCred)
/// - macOS: Keychain Services
/// - Linux: libsecret (Secret Service API)
/// - Fallback: SQLite-backed encrypted store
pub trait KeychainBackend: Send + Sync {
    /// Store a secret value associated with the given name/key.
    fn store(&self, name: &str, secret: &str) -> Result<()>;

    /// Retrieve a previously stored secret by name.
    /// Returns `None` if the entry does not exist.
    fn get(&self, name: &str) -> Result<Option<String>>;

    /// Delete a stored secret by name.
    fn delete(&self, name: &str) -> Result<()>;
}

/// SQLite-backed keychain implementation for cross-platform compatibility.
///
/// This is a **stub/fallback** implementation. In production, platform-native
/// keychains should be preferred for better security (OS-level encryption,
/// hardware isolation, access control).
///
/// ## Security Notes
/// - Secrets are stored as plaintext in the SQLite database
/// - The database file permissions should be restricted to the current user
/// - Future: migrate to platform APIs via a compile-time feature flag
pub struct SqliteKeychain {
    conn: Mutex<Connection>,
}

impl SqliteKeychain {
    /// Create a new SQLite-backed keychain.
    ///
    /// The `conn` should be a dedicated connection to the keychain database
    /// (preferably a separate file from the main app database).
    pub fn new(conn: Connection) -> Result<Self> {
        // Create the keychain table if it doesn't exist
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS keychain (
                name TEXT PRIMARY KEY,
                secret TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl KeychainBackend for SqliteKeychain {
    fn store(&self, name: &str, secret: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO keychain (name, secret, updated_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(name) DO UPDATE SET
                secret = excluded.secret,
                updated_at = datetime('now')",
            params![name, secret],
        )?;
        Ok(())
    }

    fn get(&self, name: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT secret FROM keychain WHERE name = ?1",
        )?;

        let mut rows = stmt.query_map(params![name], |row| {
            row.get::<_, String>(0)
        })?;

        match rows.next() {
            Some(Ok(secret)) => Ok(Some(secret)),
            Some(Err(e)) => Err(KeychainError::DatabaseError(e)),
            None => Ok(None),
        }
    }

    fn delete(&self, name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM keychain WHERE name = ?1",
            params![name],
        )?;

        if rows == 0 {
            return Err(KeychainError::NotFound(name.to_string()));
        }
        Ok(())
    }
}

/// Platform-native keychain implementation (stub).
///
/// This is a placeholder for platform-specific implementations.
/// When the `platform-native` feature is enabled, this will delegate to:
/// - Windows: `CredWriteW` / `CredReadW` / `CredDeleteW`
/// - macOS: Security framework Keychain Services
/// - Linux: libsecret via DBus
#[cfg(feature = "platform-native")]
pub struct PlatformKeychain;

#[cfg(feature = "platform-native")]
impl KeychainBackend for PlatformKeychain {
    fn store(&self, _name: &str, _secret: &str) -> Result<()> {
        Err(KeychainError::BackendError(
            "Platform keychain not yet implemented".into(),
        ))
    }

    fn get(&self, _name: &str) -> Result<Option<String>> {
        Err(KeychainError::BackendError(
            "Platform keychain not yet implemented".into(),
        ))
    }

    fn delete(&self, _name: &str) -> Result<()> {
        Err(KeychainError::BackendError(
            "Platform keychain not yet implemented".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_keychain() -> SqliteKeychain {
        let tmp = NamedTempFile::new().unwrap();
        let conn = Connection::open(tmp.path()).unwrap();
        SqliteKeychain::new(conn).unwrap()
    }

    #[test]
    fn test_store_and_get() {
        let keychain = create_test_keychain();
        keychain.store("vault_token", "encrypted_vault_key_here").unwrap();

        let result = keychain.get("vault_token").unwrap();
        assert_eq!(result, Some("encrypted_vault_key_here".to_string()));
    }

    #[test]
    fn test_get_nonexistent() {
        let keychain = create_test_keychain();
        let result = keychain.get("nonexistent").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_overwrite_existing() {
        let keychain = create_test_keychain();
        keychain.store("token", "old_value").unwrap();
        keychain.store("token", "new_value").unwrap();

        let result = keychain.get("token").unwrap();
        assert_eq!(result, Some("new_value".to_string()));
    }

    #[test]
    fn test_delete() {
        let keychain = create_test_keychain();
        keychain.store("temp", "temporary").unwrap();
        keychain.delete("temp").unwrap();

        let result = keychain.get("temp").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_nonexistent() {
        let keychain = create_test_keychain();
        let result = keychain.delete("ghost");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), KeychainError::NotFound(_)));
    }
}
