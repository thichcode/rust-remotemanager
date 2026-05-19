//! Test support utilities for the Hermes Remote Manager Rust backend.
#![cfg(test)]

use tempfile::TempDir;
use rusqlite::Connection;
use crate::storage::models::ConnectionCreateRequest;
use crate::storage::database::Database;

pub struct TestDatabase {
    _temp_dir: TempDir,
    pub conn: Connection,
}

impl TestDatabase {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path.to_str().unwrap()).expect("failed to create test database");
        let conn = db.into_connection();
        Self { _temp_dir: temp_dir, conn }
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

impl Default for TestDatabase {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TestConnectionRequestBuilder {
    name: String,
    r#type: String,
    host: String,
    port: i32,
    username: String,
    auth_type: String,
    credential_id: Option<String>,
    password: Option<String>,
    private_key: Option<String>,
    folder_id: Option<String>,
    tags: Option<String>,
    notes: Option<String>,
    keepalive_interval: Option<i32>,
    proxy_type: Option<String>,
    proxy_host: Option<String>,
    proxy_port: Option<i32>,
    proxy_username: Option<String>,
    color: Option<String>,
}

impl TestConnectionRequestBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            r#type: "ssh".to_string(),
            host: "127.0.0.1".to_string(),
            port: 22,
            username: "admin".to_string(),
            auth_type: "password".to_string(),
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
        }
    }

    pub fn with_host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    pub fn with_port(mut self, port: i32) -> Self {
        self.port = port;
        self
    }

    pub fn with_username(mut self, username: &str) -> Self {
        self.username = username.to_string();
        self
    }

    pub fn with_auth_type(mut self, auth_type: &str) -> Self {
        self.auth_type = auth_type.to_string();
        self
    }

    pub fn with_credential_id(mut self, credential_id: &str) -> Self {
        self.credential_id = Some(credential_id.to_string());
        self
    }

    pub fn with_password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    pub fn with_key(mut self, private_key: &str) -> Self {
        self.private_key = Some(private_key.to_string());
        self.auth_type = "key".to_string();
        self
    }

    pub fn with_folder(mut self, folder_id: &str) -> Self {
        self.folder_id = Some(folder_id.to_string());
        self
    }

    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags = Some(tags.join(","));
        self
    }

    pub fn with_notes(mut self, notes: &str) -> Self {
        self.notes = Some(notes.to_string());
        self
    }

    pub fn with_keepalive(mut self, interval: i32) -> Self {
        self.keepalive_interval = Some(interval);
        self
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }

    pub fn rdp(mut self) -> Self {
        self.r#type = "rdp".to_string();
        self.port = 3389;
        self
    }

    pub fn build(self) -> ConnectionCreateRequest {
        ConnectionCreateRequest {
            name: self.name,
            r#type: self.r#type,
            host: self.host,
            port: self.port,
            username: self.username,
            auth_type: self.auth_type,
            credential_id: self.credential_id,
            password: self.password,
            private_key: self.private_key,
            folder_id: self.folder_id,
            tags: self.tags,
            notes: self.notes,
            keepalive_interval: self.keepalive_interval,
            proxy_type: self.proxy_type,
            proxy_host: self.proxy_host,
            proxy_port: self.proxy_port,
            proxy_username: self.proxy_username,
            color: self.color,
        }
    }
}

pub fn connection_request(name: &str) -> ConnectionCreateRequest {
    TestConnectionRequestBuilder::new(name).build()
}

pub fn ssh_connection(name: &str, host: &str, username: &str) -> ConnectionCreateRequest {
    TestConnectionRequestBuilder::new(name)
        .with_host(host)
        .with_username(username)
        .build()
}

pub fn rdp_connection(name: &str, host: &str, username: &str) -> ConnectionCreateRequest {
    TestConnectionRequestBuilder::new(name)
        .rdp()
        .with_host(host)
        .with_username(username)
        .build()
}