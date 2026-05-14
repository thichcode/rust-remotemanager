use crate::error::AppResult;
use rusqlite::Connection;
use std::collections::HashMap;

/// Manages application settings persisted in the database.
#[derive(Clone, Debug)]
pub struct SettingsManager {
    settings: HashMap<String, String>,
}

impl SettingsManager {
    /// Load all settings from the database.
    pub fn new(conn: &Connection) -> AppResult<Self> {
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })?;

        let mut settings = HashMap::new();
        for row in rows {
            let (key, value) = row?;
            settings.insert(key, value);
        }

        Ok(Self { settings })
    }

    /// Get a setting value by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.settings.get(key).map(|s| s.as_str())
    }

    /// Set a setting value (persisted immediately).
    pub fn set(&mut self, conn: &Connection, key: &str, value: &str) -> AppResult<()> {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, value],
        )?;
        self.settings.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Remove a setting.
    pub fn remove(&mut self, conn: &Connection, key: &str) -> AppResult<()> {
        conn.execute("DELETE FROM settings WHERE key = ?1", rusqlite::params![key])?;
        self.settings.remove(key);
        Ok(())
    }

    /// Get all settings as a HashMap.
    pub fn get_all(&self) -> HashMap<String, String> {
        self.settings.clone()
    }
}
