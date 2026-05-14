use rusqlite::{Connection, Result};

/// Run all pending migrations on the database.
///
/// Migrations are tracked in a `_migrations` table and applied
/// sequentially based on version numbers.
pub fn run(conn: &Connection) -> Result<()> {
    // Create migration tracking table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL DEFAULT '',
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    // Get the latest applied migration version
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Apply migrations in order
    if current_version < 1 {
        log_migration(conn, 1, "Creating initial schema")?;
        conn.execute_batch(MIGRATION_V1)?;
    }

    if current_version < 2 {
        log_migration(conn, 2, "Adding session_logs, settings, snippets")?;
        conn.execute_batch(MIGRATION_V2)?;
    }

    if current_version < 3 {
        log_migration(conn, 3, "Creating performance indexes")?;
        conn.execute_batch(MIGRATION_V3)?;
    }

    Ok(())
}

fn log_migration(conn: &Connection, version: i32, description: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO _migrations (version, description) VALUES (?1, ?2)",
        rusqlite::params![version, description],
    )?;
    Ok(())
}

/// V1: Core tables — folders, connections, credentials
const MIGRATION_V1: &str = "
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id TEXT REFERENCES folders(id) ON DELETE CASCADE,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS connections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('ssh','rdp','serial','telnet','vnc')),
    folder_id TEXT REFERENCES folders(id) ON DELETE SET NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL,
    username TEXT NOT NULL,
    credential_id TEXT REFERENCES credentials(id) ON DELETE SET NULL,
    auth_type TEXT NOT NULL DEFAULT 'password',
    proxy_type TEXT,
    proxy_host TEXT,
    proxy_port INTEGER,
    proxy_username TEXT,
    tags TEXT,
    notes TEXT,
    startup_commands TEXT,
    keepalive_interval INTEGER DEFAULT 30,
    sort_order INTEGER DEFAULT 0,
    is_favorite INTEGER DEFAULT 0,
    color TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS credentials (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    auth_type TEXT NOT NULL CHECK(auth_type IN ('password','key','agent')),
    username TEXT,
    encrypted_password BLOB,
    key_type TEXT,
    encrypted_private_key BLOB,
    key_path TEXT,
    passphrase_protected INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
";

/// V2: Session logs, settings, snippets
const MIGRATION_V2: &str = "
CREATE TABLE IF NOT EXISTS session_logs (
    id TEXT PRIMARY KEY,
    connection_id TEXT REFERENCES connections(id) ON DELETE CASCADE,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    bytes_sent INTEGER DEFAULT 0,
    bytes_received INTEGER DEFAULT 0,
    log_path TEXT
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS snippets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    command TEXT NOT NULL,
    category TEXT,
    shortcut TEXT,
    sort_order INTEGER DEFAULT 0
);
";

/// V3: Performance indexes
const MIGRATION_V3: &str = "
CREATE INDEX IF NOT EXISTS idx_connections_folder ON connections(folder_id);
CREATE INDEX IF NOT EXISTS idx_connections_type ON connections(type);
CREATE INDEX IF NOT EXISTS idx_session_logs_connection ON session_logs(connection_id);
CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders(parent_id);
CREATE INDEX IF NOT EXISTS idx_credentials_auth_type ON credentials(auth_type);
";
