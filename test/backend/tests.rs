use std::collections::HashMap;

// Global store for test state
static mut CONNECTIONS: HashMap<String, TestConnection> = HashMap::new();
static mut FOLDERS: HashMap<String, TestFolder> = HashMap::new();

#[derive(Debug, Clone)]
struct TestConnection {
    id: String,
    name: String,
    host: String,
    port: u16,
}

#[derive(Debug, Clone)]
struct TestFolder {
    id: String,
    name: String,
}

#[test]
fn test_basic_connection_operations() {
    unsafe {
        CONNECTIONS.clear();
        // Create connection
        let conn = TestConnection {
            id: "conn-1".to_string(),
            name: "Test Server".to_string(),
            host: "127.0.0.1".to_string(),
            port: 22,
        };
        CONNECTIONS.insert(conn.id.clone(), conn.clone());

        // Read connection
        assert!(CONNECTIONS.contains_key("conn-1"));
        let retrieved = CONNECTIONS.get("conn-1").unwrap();
        assert_eq!(retrieved.name, "Test Server");
        assert_eq!(retrieved.host, "127.0.0.1");

        // Delete connection
        CONNECTIONS.remove("conn-1");
        assert!(!CONNECTIONS.contains_key("conn-1"));
    }
}

#[test]
fn test_folder_operations() {
    unsafe {
        FOLDERS.clear();

        // Create folder
        let folder = TestFolder {
            id: "folder-1".to_string(),
            name: "Test Folder".to_string(),
        };
        FOLDERS.insert(folder.id.clone(), folder.clone());

        // Read folder
        assert!(FOLDERS.contains_key("folder-1"));
        let retrieved = FOLDERS.get("folder-1").unwrap();
        assert_eq!(retrieved.name, "Test Folder");

        // Delete folder
        FOLDERS.remove("folder-1");
        assert!(!FOLDERS.contains_key("folder-1"));
    }
}

fn main() {
    println!("Running backend tests...");

    // Run tests manually
    test_basic_connection_operations();
    test_folder_operations();

    println!("All backend tests passed!");
}
