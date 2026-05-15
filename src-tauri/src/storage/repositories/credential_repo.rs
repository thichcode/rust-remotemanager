use crate::error::AppResult;
use crate::storage::models::Credential;
use chrono::Utc;
use rusqlite::{params, Connection as DbConnection};
use uuid::Uuid;

/// Repository for managing stored credentials (encrypted secrets).
pub struct CredentialRepository<'a> {
    conn: &'a DbConnection,
}

impl<'a> CredentialRepository<'a> {
    /// Create a new repository bound to the given database connection.
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }

    /// List all credentials, ordered by name.
    pub fn list(&self) -> AppResult<Vec<Credential>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, auth_type, username,
                    encrypted_password, key_type, encrypted_private_key,
                    key_path, passphrase_protected,
                    created_at, updated_at
             FROM credentials
             ORDER BY name ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Credential {
                id: row.get("id")?,
                name: row.get("name")?,
                auth_type: row.get("auth_type")?,
                username: row.get("username")?,
                encrypted_password: row.get("encrypted_password")?,
                key_type: row.get("key_type")?,
                encrypted_private_key: row.get("encrypted_private_key")?,
                key_path: row.get("key_path")?,
                passphrase_protected: row.get::<_, i32>("passphrase_protected")? != 0,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;

        let mut credentials = Vec::new();
        for row in rows {
            credentials.push(row?);
        }
        Ok(credentials)
    }

    #[allow(dead_code)]
    /// Get a single credential by ID.
    pub fn get_by_id(&self, id: &str) -> AppResult<Option<Credential>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, auth_type, username,
                    encrypted_password, key_type, encrypted_private_key,
                    key_path, passphrase_protected,
                    created_at, updated_at
             FROM credentials WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Credential {
                id: row.get("id")?,
                name: row.get("name")?,
                auth_type: row.get("auth_type")?,
                username: row.get("username")?,
                encrypted_password: row.get("encrypted_password")?,
                key_type: row.get("key_type")?,
                encrypted_private_key: row.get("encrypted_private_key")?,
                key_path: row.get("key_path")?,
                passphrase_protected: row.get::<_, i32>("passphrase_protected")? != 0,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?;

        match rows.next() {
            Some(Ok(cred)) => Ok(Some(cred)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    /// Create a new credential, generating UUID and timestamps.
    pub fn create(&self, cred: Credential) -> AppResult<Credential> {
        let now = Utc::now().to_rfc3339();
        let id = if cred.id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            cred.id.clone()
        };

        self.conn.execute(
            "INSERT INTO credentials (
                id, name, auth_type, username,
                encrypted_password, key_type, encrypted_private_key,
                key_path, passphrase_protected,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                id,
                cred.name,
                cred.auth_type,
                cred.username,
                cred.encrypted_password,
                cred.key_type,
                cred.encrypted_private_key,
                cred.key_path,
                cred.passphrase_protected as i32,
                now,
                now,
            ],
        )?;

        Ok(Credential {
            id,
            name: cred.name,
            auth_type: cred.auth_type,
            username: cred.username,
            encrypted_password: cred.encrypted_password,
            key_type: cred.key_type,
            encrypted_private_key: cred.encrypted_private_key,
            key_path: cred.key_path,
            passphrase_protected: cred.passphrase_protected,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    #[allow(dead_code)]
    /// Update an existing credential. Returns true if a row was modified.
    pub fn update(&self, cred: Credential) -> AppResult<bool> {
        let now = Utc::now().to_rfc3339();
        let rows = self.conn.execute(
            "UPDATE credentials SET
                name = ?1, auth_type = ?2, username = ?3,
                encrypted_password = ?4, key_type = ?5,
                encrypted_private_key = ?6, key_path = ?7,
                passphrase_protected = ?8,
                updated_at = ?9
             WHERE id = ?10",
            params![
                cred.name,
                cred.auth_type,
                cred.username,
                cred.encrypted_password,
                cred.key_type,
                cred.encrypted_private_key,
                cred.key_path,
                cred.passphrase_protected as i32,
                now,
                cred.id,
            ],
        )?;
        Ok(rows > 0)
    }

    /// Delete a credential by ID. Returns true if a row was deleted.
    pub fn delete(&self, id: &str) -> AppResult<bool> {
        let rows = self
            .conn
            .execute("DELETE FROM credentials WHERE id = ?1", params![id])?;
        Ok(rows > 0)
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
        let repo = CredentialRepository::new(db.get_conn());

        let cred = Credential {
            id: String::new(),
            name: "My SSH Key".into(),
            auth_type: "key".into(),
            username: Some("ubuntu".into()),
            encrypted_password: None,
            key_type: Some("ed25519".into()),
            encrypted_private_key: Some(vec![1, 2, 3, 4]),
            key_path: Some("/home/user/.ssh/id_ed25519".into()),
            passphrase_protected: true,
            created_at: String::new(),
            updated_at: String::new(),
        };

        let created = repo.create(cred).unwrap();
        assert_eq!(created.name, "My SSH Key");
        assert!(!created.id.is_empty());

        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_get_by_id() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = CredentialRepository::new(db.get_conn());

        let cred = Credential {
            id: String::new(),
            name: "Test Cred".into(),
            auth_type: "password".into(),
            username: Some("admin".into()),
            encrypted_password: Some(vec![10, 20, 30]),
            key_type: None,
            encrypted_private_key: None,
            key_path: None,
            passphrase_protected: false,
            created_at: String::new(),
            updated_at: String::new(),
        };

        let created = repo.create(cred).unwrap();
        let found = repo.get_by_id(&created.id).unwrap().unwrap();
        assert_eq!(found.name, "Test Cred");
        assert_eq!(found.username, Some("admin".into()));
    }

    #[test]
    fn test_update() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = CredentialRepository::new(db.get_conn());

        let cred = Credential {
            id: String::new(),
            name: "Original".into(),
            auth_type: "password".into(),
            username: None,
            encrypted_password: None,
            key_type: None,
            encrypted_private_key: None,
            key_path: None,
            passphrase_protected: false,
            created_at: String::new(),
            updated_at: String::new(),
        };

        let mut created = repo.create(cred).unwrap();
        created.name = "Updated".into();
        assert!(repo.update(created.clone()).unwrap());

        let found = repo.get_by_id(&created.id).unwrap().unwrap();
        assert_eq!(found.name, "Updated");
    }

    #[test]
    fn test_delete() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        crate::storage::migrations::run(db.get_conn()).unwrap();
        let repo = CredentialRepository::new(db.get_conn());

        let cred = Credential {
            id: String::new(),
            name: "To Delete".into(),
            auth_type: "password".into(),
            username: None,
            encrypted_password: None,
            key_type: None,
            encrypted_private_key: None,
            key_path: None,
            passphrase_protected: false,
            created_at: String::new(),
            updated_at: String::new(),
        };

        let created = repo.create(cred).unwrap();
        assert!(repo.delete(&created.id).unwrap());
        assert!(repo.get_by_id(&created.id).unwrap().is_none());
    }
}
