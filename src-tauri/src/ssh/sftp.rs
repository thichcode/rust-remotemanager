use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpFileInfo {
    pub name: String,
    pub path: String,
    pub size: i64,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub permissions: String,
    pub modified: String,
}

/// Wraps an ssh2::Sftp session for file operations over an existing SSH connection.
pub struct SftpSession {
    pub sftp: ssh2::Sftp,
}

impl SftpSession {
    /// Create a new SFTP session from an existing SSH session.
    pub fn new(session: &ssh2::Session) -> Result<Self, ssh2::Error> {
        let sftp = session.sftp()?;
        Ok(Self { sftp })
    }

    /// List directory contents at the given path.
    pub fn list_dir(&self, path: &str) -> AppResult<Vec<SftpFileInfo>> {
        let path_path = Path::new(path);
        let entries = self.sftp.readdir(path_path)?;

        let mut files = Vec::with_capacity(entries.len());
        for (entry_path, stat) in entries {
            let name = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let is_dir = stat.is_dir();
            let is_symlink = stat.file_type().is_symlink();
            let permissions = format!("{:o}", stat.perm.unwrap_or(0));
            let modified = stat
                .mtime
                .map(|t| {
                    chrono::DateTime::from_timestamp(t as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                })
                .unwrap_or_default();

            files.push(SftpFileInfo {
                name,
                path: entry_path.to_string_lossy().to_string(),
                size: stat.size.unwrap_or(0) as i64,
                is_dir,
                is_symlink,
                permissions,
                modified,
            });
        }

        // Sort: directories first, then alphabetically
        files.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                if a.is_dir {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        Ok(files)
    }

    /// Read a remote file's contents into memory.
    pub fn read_file(&self, path: &str) -> AppResult<Vec<u8>> {
        let path_path = Path::new(path);
        let mut file = self.sftp.open(path_path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data)
    }

    /// Write data to a remote file (creates or overwrites).
    pub fn write_file(&self, path: &str, data: &[u8]) -> AppResult<()> {
        let path_path = Path::new(path);
        use std::io::Write;
        let mut file = self.sftp.create(path_path)?;
        file.write_all(data)?;
        Ok(())
    }

    /// Create a directory on the remote server.
    pub fn create_dir(&self, path: &str) -> AppResult<()> {
        let path_path = Path::new(path);
        self.sftp.mkdir(path_path, 0o755)?;
        Ok(())
    }

    /// Remove a file on the remote server.
    pub fn remove_file(&self, path: &str) -> AppResult<()> {
        let path_path = Path::new(path);
        self.sftp.unlink(path_path)?;
        Ok(())
    }

    /// Remove an empty directory on the remote server.
    pub fn remove_dir(&self, path: &str) -> AppResult<()> {
        let path_path = Path::new(path);
        self.sftp.rmdir(path_path)?;
        Ok(())
    }

    /// Rename (or move) a remote file/directory.
    pub fn rename(&self, old: &str, new: &str) -> AppResult<()> {
        let old_path = Path::new(old);
        let new_path = Path::new(new);
        self.sftp.rename(old_path, new_path, None)?;
        Ok(())
    }

    /// Get file/directory info at the given path.
    pub fn stat(&self, path: &str) -> AppResult<SftpFileInfo> {
        let path_path = Path::new(path);
        let stat = self.sftp.stat(path_path)?;

        let name = path_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
        let is_dir = stat.is_dir();
        let is_symlink = stat.file_type().is_symlink();
        let permissions = format!("{:o}", stat.perm.unwrap_or(0));
        let modified = stat
            .mtime
            .map(|t| {
                chrono::DateTime::from_timestamp(t as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        Ok(SftpFileInfo {
            name,
            path: path_path.to_string_lossy().to_string(),
            size: stat.size.unwrap_or(0) as i64,
            is_dir,
            is_symlink,
            permissions,
            modified,
        })
    }
}
