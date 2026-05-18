use crate::AppState;
use crate::error::AppResult;
use tauri::State;
use std::collections::HashMap;

/// Get all application settings as a key-value map.
#[tauri::command]
pub fn get_settings(state: State<AppState>) -> HashMap<String, String> {
    let settings = state.settings.lock();
    settings.get_all()
}

/// Update a single application setting.
#[tauri::command]
pub fn update_setting(
    state: State<AppState>,
    key: String,
    value: String,
) -> AppResult<()> {
    let db = state.db.lock();
    let mut settings = state.settings.lock();
    settings.set(&db, &key, &value)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::storage::database::Database;
    use crate::storage::migrations;
    use crate::settings::SettingsManager;
    use tempfile::NamedTempFile;

    #[test]
    fn test_settings_new_empty() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        migrations::run(db.get_conn()).unwrap();
        let settings = SettingsManager::new(db.get_conn()).unwrap();
        let all = settings.get_all();
        assert!(all.is_empty());
    }

    #[test]
    fn test_settings_set_and_get() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        migrations::run(db.get_conn()).unwrap();
        let mut settings = SettingsManager::new(db.get_conn()).unwrap();
        settings.set(db.get_conn(), "theme", "dark").unwrap();
        assert_eq!(settings.get("theme"), Some("dark"));
    }

    #[test]
    fn test_settings_overwrite() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        migrations::run(db.get_conn()).unwrap();
        let mut settings = SettingsManager::new(db.get_conn()).unwrap();
        settings.set(db.get_conn(), "key", "value1").unwrap();
        settings.set(db.get_conn(), "key", "value2").unwrap();
        assert_eq!(settings.get("key"), Some("value2"));
    }

    #[test]
    fn test_settings_remove() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        migrations::run(db.get_conn()).unwrap();
        let mut settings = SettingsManager::new(db.get_conn()).unwrap();
        settings.set(db.get_conn(), "temp", "value").unwrap();
        settings.remove(db.get_conn(), "temp").unwrap();
        assert!(settings.get("temp").is_none());
    }

    #[test]
    fn test_settings_get_nonexistent() {
        let tmp = NamedTempFile::new().unwrap();
        let db = Database::new(tmp.path().to_str().unwrap()).unwrap();
        migrations::run(db.get_conn()).unwrap();
        let settings = SettingsManager::new(db.get_conn()).unwrap();
        assert!(settings.get("nonexistent").is_none());
    }

    #[test]
    fn test_settings_persistence() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        {
            let db = Database::new(&path).unwrap();
            migrations::run(db.get_conn()).unwrap();
            let mut settings = SettingsManager::new(db.get_conn()).unwrap();
            settings.set(db.get_conn(), "port", "8080").unwrap();
        }
        let db = Database::new(&path).unwrap();
        let settings = SettingsManager::new(db.get_conn()).unwrap();
        assert_eq!(settings.get("port"), Some("8080"));
    }
}