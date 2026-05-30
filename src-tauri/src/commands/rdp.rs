use crate::error::{AppError, AppResult};
use serde::Deserialize;
use std::io::Write;

#[derive(Debug, Deserialize)]
pub struct RdpConfig {
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub domain: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fullscreen: Option<bool>,
}

fn build_rdp_content(config: &RdpConfig) -> String {
    let port = config.port.unwrap_or(3389);
    let username = config.username.as_deref().unwrap_or("");
    let width = config.width.unwrap_or(1280);
    let height = config.height.unwrap_or(720);
    let fullscreen = config.fullscreen.unwrap_or(false);

    let mut rdp_content = String::new();
    rdp_content.push_str(&format!("full address:s:{}:{}\n", config.host, port));
    rdp_content.push_str("prompt for credentials:i:1\n");
    rdp_content.push_str("authentication level:i:2\n");
    rdp_content.push_str("session bpp:i:32\n");
    rdp_content.push_str("networkautodetect:i:1\n");
    rdp_content.push_str("bandwidthautodetect:i:1\n");
    rdp_content.push_str("connection type:i:7\n");
    rdp_content.push_str("audiomode:i:0\n");
    rdp_content.push_str("redirectprinters:i:0\n");
    rdp_content.push_str("redirectcomports:i:0\n");
    rdp_content.push_str("redirectsmartcards:i:1\n");
    rdp_content.push_str("redirectclipboard:i:1\n");
    rdp_content.push_str("redirectposdevices:i:0\n");
    rdp_content.push_str("redirectdirectx:i:1\n");
    rdp_content.push_str("displayconnectionbar:i:1\n");

    if fullscreen {
        rdp_content.push_str("screen mode id:i:2\n");
    } else {
        rdp_content.push_str("screen mode id:i:1\n");
        rdp_content.push_str(&format!("desktopwidth:i:{}\n", width));
        rdp_content.push_str(&format!("desktopheight:i:{}\n", height));
    }

    if !username.is_empty() {
        rdp_content.push_str(&format!("username:s:{}\n", username));
    }
    if let Some(ref domain) = config.domain {
        rdp_content.push_str(&format!("domain:s:{}\n", domain));
    }

    rdp_content.push_str("allow desktop composition:i:1\n");
    rdp_content.push_str("allow font smoothing:i:1\n");
    rdp_content.push_str("disable wallpaper:i:0\n");
    rdp_content.push_str("disable full window drag:i:0\n");
    rdp_content.push_str("disable menu anims:i:0\n");
    rdp_content.push_str("disable themes:i:0\n");
    rdp_content.push_str("disable cursor setting:i:0\n");

    rdp_content
}

#[tauri::command]
pub fn connect_rdp(config: RdpConfig) -> AppResult<String> {
    let rdp_content = build_rdp_content(&config);

    let temp_dir = std::env::temp_dir();
    let file_name = format!("hermes-rdp-{}.rdp", uuid::Uuid::new_v4());
    let rdp_path = temp_dir.join(&file_name);

    let mut file =
        std::fs::File::create(&rdp_path).map_err(|e| AppError::Io(format!("create rdp file: {}", e)))?;
    file.write_all(rdp_content.as_bytes())
        .map_err(|e| AppError::Io(format!("write rdp file: {}", e)))?;
    file.flush()
        .map_err(|e| AppError::Io(format!("flush rdp file: {}", e)))?;
    drop(file);

    let rdp_path_str = rdp_path.to_string_lossy().to_string();

    let child = std::process::Command::new("mstsc")
        .arg(&rdp_path_str)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| AppError::Io(format!("launch mstsc: {}", e)))?;

    let pid = child.id();

    // Schedule file cleanup after a short delay
    let cleanup_path = rdp_path_str.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let _ = std::fs::remove_file(&cleanup_path);
    });

    Ok(serde_json::json!({
        "pid": pid,
        "rdp_file": rdp_path_str,
    })
    .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_rdp_content_with_defaults() {
        let config = RdpConfig {
            host: "127.0.0.1".into(),
            port: None,
            username: None,
            domain: None,
            width: None,
            height: None,
            fullscreen: None,
        };

        let content = build_rdp_content(&config);
        assert!(content.contains("full address:s:127.0.0.1:3389"));
        assert!(content.contains("screen mode id:i:1"));
        assert!(content.contains("desktopwidth:i:1280"));
        assert!(content.contains("desktopheight:i:720"));
    }

    #[test]
    fn builds_rdp_content_fullscreen_and_identity() {
        let config = RdpConfig {
            host: "rdp.example.com".into(),
            port: Some(3390),
            username: Some("admin".into()),
            domain: Some("CORP".into()),
            width: Some(1920),
            height: Some(1080),
            fullscreen: Some(true),
        };

        let content = build_rdp_content(&config);
        assert!(content.contains("full address:s:rdp.example.com:3390"));
        assert!(content.contains("screen mode id:i:2"));
        assert!(!content.contains("desktopwidth:i:1920"));
        assert!(content.contains("username:s:admin"));
        assert!(content.contains("domain:s:CORP"));
    }
}
