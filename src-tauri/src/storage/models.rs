use serde::{Deserialize, Serialize};

/// Represents a folder in the connection tree hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Represents a remote connection (SSH, RDP, Serial, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub folder_id: Option<String>,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub credential_id: Option<String>,
    pub auth_type: String,
    pub proxy_type: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i32>,
    pub proxy_username: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub startup_commands: Option<String>,
    pub keepalive_interval: Option<i32>,
    pub is_favorite: bool,
    pub color: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Represents stored authentication credentials with encrypted secrets.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    pub id: String,
    pub name: String,
    pub auth_type: String,
    pub username: Option<String>,
    pub encrypted_password: Option<Vec<u8>>,
    pub key_type: Option<String>,
    pub encrypted_private_key: Option<Vec<u8>>,
    pub key_path: Option<String>,
    pub passphrase_protected: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Represents a single terminal session log entry.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    pub id: String,
    pub connection_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub bytes_sent: i64,
    pub bytes_received: i64,
    pub log_path: Option<String>,
}

/// Request payload for creating a new connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCreateRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub auth_type: String,
    pub credential_id: Option<String>,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub folder_id: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub keepalive_interval: Option<i32>,
    pub proxy_type: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i32>,
    pub proxy_username: Option<String>,
    pub color: Option<String>,
}

/// Key-value settings entry.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

/// Represents a saved command snippet for quick reuse.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub name: String,
    pub command: String,
    pub category: Option<String>,
    pub shortcut: Option<String>,
    pub sort_order: i32,
}
