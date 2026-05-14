use serde::{Deserialize, Serialize};
use ssh2::Channel;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Configuration for establishing an SSH connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub key_content: Option<String>,
    pub passphrase: Option<String>,
    pub keepalive_interval: Option<u32>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<u16>,
    pub proxy_username: Option<String>,
}

/// State of an SSH session, returned to the frontend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionState {
    Disconnected,
    Connecting,
    #[serde(rename = "Connected")]
    Connected { cols: u16, rows: u16 },
    Error(String),
}

/// Commands that can be sent to a session's I/O thread.
#[derive(Debug)]
pub enum ChannelCommand {
    Input(Vec<u8>),
    Resize(u16, u16),
    Close,
}

/// Represents an active SSH terminal session with real SSH connection.
pub struct SshSession {
    pub id: String,
    pub config: SshConfig,
    pub state: Arc<Mutex<SessionState>>,
    pub running: Arc<AtomicBool>,
    pub cmd_tx: Option<Sender<ChannelCommand>>,
    pub thread_handle: Option<thread::JoinHandle<()>>,
}

impl SshSession {
    fn new(id: String, config: SshConfig) -> Self {
        Self {
            id,
            config,
            state: Arc::new(Mutex::new(SessionState::Disconnected)),
            running: Arc::new(AtomicBool::new(false)),
            cmd_tx: None,
            thread_handle: None,
        }
    }
}

/// Manages all active SSH sessions with real connections.
pub struct SessionManager {
    sessions: HashMap<String, SshSession>,
}

impl SessionManager {
    /// Create a new empty session manager.
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Initiate a new SSH connection. This spawns a background thread that
    /// performs the TCP/SSH handshake, authenticates, starts a shell, and
    /// then streams output via Tauri events. Returns the session ID immediately.
    pub fn connect(
        &mut self,
        config: SshConfig,
        app_handle: AppHandle,
        session_id: String,
    ) -> String {
        let id = session_id.clone();

        let state = Arc::new(Mutex::new(SessionState::Connecting));
        let running = Arc::new(AtomicBool::new(true));

        let (cmd_tx, cmd_rx) = mpsc::channel::<ChannelCommand>();

        let session = SshSession {
            id: id.clone(),
            config: config.clone(),
            state: state.clone(),
            running: running.clone(),
            cmd_tx: Some(cmd_tx),
            thread_handle: None,
        };

        // Insert a placeholder; we'll set the thread handle after spawning.
        self.sessions.insert(id.clone(), session);

        let thread_id = id.clone();
        let thread_running = running.clone();
        let thread_state = state.clone();

        let handle = thread::Builder::new()
            .name(format!("ssh-io-{}", &thread_id[..8.min(thread_id.len())]))
            .spawn(move || {
                run_ssh_session(
                    &thread_id,
                    &config,
                    app_handle,
                    thread_state,
                    thread_running,
                    cmd_rx,
                );
            })
            .expect("failed to spawn SSH I/O thread");

        // Update the session with the thread handle.
        if let Some(s) = self.sessions.get_mut(&id) {
            s.thread_handle = Some(handle);
        }

        id
    }

    /// Get a reference to a session by ID.
    pub fn get(&self, id: &str) -> Option<&SshSession> {
        self.sessions.get(id)
    }

    /// Get a mutable reference to a session by ID.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SshSession> {
        self.sessions.get_mut(id)
    }

    /// Disconnect and remove a session by ID. Sends Close to the I/O thread
    /// and waits for it to finish.
    pub fn remove(&mut self, id: &str) -> Option<SshSession> {
        if let Some(mut session) = self.sessions.remove(id) {
            session.running.store(false, Ordering::SeqCst);
            if let Some(ref tx) = session.cmd_tx {
                let _ = tx.send(ChannelCommand::Close);
            }
            if let Some(handle) = session.thread_handle.take() {
                let _ = handle.join();
            }
            Some(session)
        } else {
            None
        }
    }

    /// Send input data to an active session's SSH channel.
    pub fn send_input(&self, id: &str, data: &str) -> Result<(), String> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| format!("Session '{}' not found", id))?;
        if let Some(tx) = &session.cmd_tx {
            tx.send(ChannelCommand::Input(data.as_bytes().to_vec()))
                .map_err(|_| "Failed to send input: channel closed".to_string())?;
            Ok(())
        } else {
            Err("Session has no command channel".to_string())
        }
    }

    /// Resize the terminal PTY for an active session.
    pub fn resize(&self, id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let session = self
            .sessions
            .get(id)
            .ok_or_else(|| format!("Session '{}' not found", id))?;
        if let Some(tx) = &session.cmd_tx {
            tx.send(ChannelCommand::Resize(cols, rows))
                .map_err(|_| "Failed to send resize: channel closed".to_string())?;
            Ok(())
        } else {
            Err("Session has no command channel".to_string())
        }
    }

    /// Get the current state of a session.
    pub fn get_state(&self, id: &str) -> Option<SessionState> {
        self.sessions
            .get(id)
            .map(|s| s.state.lock().unwrap().clone())
    }

    /// List all active session IDs.
    pub fn list(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }

    /// Return the number of active sessions.
    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }
}

// ─────────────────────────────────────────────────────────────────────
// Internal: the real SSH connection + I/O loop running on a background
// thread for each session.
// ─────────────────────────────────────────────────────────────────────

fn run_ssh_session(
    session_id: &str,
    config: &SshConfig,
    app_handle: AppHandle,
    state: Arc<Mutex<SessionState>>,
    running: Arc<AtomicBool>,
    cmd_rx: Receiver<ChannelCommand>,
) {
    let id = session_id.to_string();
    let result = establish_connection(config);

    let mut channel = match result {
        Ok(ch) => {
            *state.lock().unwrap() = SessionState::Connected {
                cols: 80,
                rows: 24,
            };
            ch
        }
        Err(e) => {
            *state.lock().unwrap() = SessionState::Error(e.clone());
            // Emit error event so the frontend knows the connection failed.
            let _ = app_handle.emit(
                "terminal:error",
                serde_json::json!({
                    "id": id,
                    "error": e,
                }),
            );
            return;
        }
    };

    // Attempt to set non-blocking on the channel so reads don't hang
    // indefinitely.  (Channel inherits blocking mode from the session's
    // underlying TCP socket, which defaults to blocking.  We leave it
    // blocking and use try_recv + a short sleep to poll for commands.)

    let mut buf = [0u8; 4096];

    // ── Main I/O loop ──────────────────────────────────────────────
    loop {
        // 1. Drain any pending commands (non-blocking).
        loop {
            match cmd_rx.try_recv() {
                Ok(ChannelCommand::Input(data)) => {
                    if let Err(err) = channel.write_all(&data) {
                        tracing::error!("[{}] write error: {}", id, err);
                        break;
                    }
                    let _ = channel.flush();
                }
                Ok(ChannelCommand::Resize(cols, rows)) => {
                    if let Err(err) = channel.request_pty_size(u32::from(cols), u32::from(rows), None, None) {
                        tracing::warn!("[{}] resize error: {}", id, err);
                    } else {
                        *state.lock().unwrap() = SessionState::Connected { cols, rows };
                    }
                }
                Ok(ChannelCommand::Close) | Err(TryRecvError::Disconnected) => {
                    // Graceful shutdown.
                    let _ = channel.send_eof();
                    let _ = channel.wait_close();
                    *state.lock().unwrap() = SessionState::Disconnected;
                    return;
                }
                Err(TryRecvError::Empty) => break,
            }
        }

        if !running.load(Ordering::SeqCst) {
            break;
        }

        // 2. Read stdout from the channel (non-blocking).
        loop {
            match channel.read(&mut buf) {
                Ok(0) => {
                    // EOF from remote side — shell likely exited.
                    tracing::info!("[{}] EOF on stdout", id);
                    let _ = app_handle.emit(
                        "terminal:exit",
                        serde_json::json!({ "id": id }),
                    );
                    *state.lock().unwrap() = SessionState::Disconnected;
                    return;
                }
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app_handle.emit(
                        "terminal:output",
                        serde_json::json!({ "id": id, "data": data }),
                    );
                    // Try to read more in the same batch.
                    continue;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No more data right now.
                }
                Err(e) => {
                    tracing::error!("[{}] read error: {}", id, e);
                    let _ = app_handle.emit(
                        "terminal:error",
                        serde_json::json!({ "id": id, "error": e.to_string() }),
                    );
                    *state.lock().unwrap() = SessionState::Error(e.to_string());
                    return;
                }
            }
            break;
        }

        // 3. Read stderr from the channel (non-blocking).
        loop {
            // `channel.stderr()` returns a wrapper that borrows `channel`
            // mutably — we scope it so the borrow is dropped afterwards.
            let mut stderr = channel.stderr();
            match stderr.read(&mut buf) {
                Ok(0) => {}
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app_handle.emit(
                        "terminal:stderr",
                        serde_json::json!({ "id": id, "data": data }),
                    );
                    continue;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    tracing::warn!("[{}] stderr read error: {}", id, e);
                }
            }
            break;
        }

        // 4. Check if the remote side has closed the channel.
        if channel.eof() {
            tracing::info!("[{}] channel eof", id);
            let _ = app_handle.emit(
                "terminal:exit",
                serde_json::json!({ "id": id }),
            );
            *state.lock().unwrap() = SessionState::Disconnected;
            return;
        }

        // 5. Brief sleep to prevent busy-waiting when idle.
        thread::sleep(Duration::from_millis(10));
    }

    // Cleanup when `running` was set to false externally.
    *state.lock().unwrap() = SessionState::Disconnected;
    let _ = channel.send_eof();
    let _ = channel.wait_close();
}

/// Perform the full SSH connection sequence:
/// TCP connect → SSH handshake → authenticate → open channel → PTY → shell.
fn establish_connection(config: &SshConfig) -> Result<Channel, String> {
    // 1. TCP connect
    let addr = format!("{}:{}", config.host, config.port);
    let tcp = TcpStream::connect(&addr)
        .map_err(|e| format!("TCP connection to {} failed: {}", addr, e))?;
    tcp.set_nonblocking(false)
        .map_err(|e| format!("Failed to set TCP stream blocking: {}", e))?;

    // 2. Create SSH session and handshake
    let mut session = ssh2::Session::new()
        .map_err(|e| format!("Failed to create SSH session: {}", e))?;
    session.set_tcp_stream(tcp);
    session.handshake()
        .map_err(|e| format!("SSH handshake with {} failed: {}", config.host, e))?;

    // 3. Authenticate
    match config.auth_type.as_str() {
        "password" => {
            let password = config.password.as_deref().unwrap_or("");
            session
                .userauth_password(&config.username, password)
                .map_err(|e| format!("Password authentication failed: {}", e))?;
        }
        "key" => {
            let key_path = config.key_path.as_deref().unwrap_or("");
            let passphrase = config.passphrase.as_deref();
            session
                .userauth_pubkey_file(
                    &config.username,
                    None,
                    std::path::Path::new(key_path),
                    passphrase,
                )
                .map_err(|e| format!("Public key authentication failed: {}", e))?;
        }
        "agent" => {
            session
                .userauth_agent(&config.username)
                .map_err(|e| format!("SSH agent authentication failed: {}", e))?;
        }
        other => return Err(format!("Unknown authentication type: '{}'", other)),
    }

    // 4. Open channel, request PTY, start shell
    let mut channel = session
        .channel_session()
        .map_err(|e| format!("Failed to open SSH channel: {}", e))?;

    channel
        .request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
        .map_err(|e| format!("Failed to request PTY: {}", e))?;

    channel
        .shell()
        .map_err(|e| format!("Failed to start shell: {}", e))?;

    Ok(channel)
}

// ─────────────────────────────────────────────────────────────────────
// Tests  (adapted from original stub; no real SSH — just manager tests)
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> SshConfig {
        SshConfig {
            host: "192.168.1.1".into(),
            port: 22,
            username: "admin".into(),
            auth_type: "password".into(),
            password: Some("secret".into()),
            key_path: None,
            key_content: None,
            passphrase: None,
            keepalive_interval: Some(30),
            proxy_host: None,
            proxy_port: None,
            proxy_username: None,
        }
    }

    #[test]
    fn test_create_and_get() {
        let mut mgr = SessionManager::new();
        let config = make_config();
        // We just test that we can create an entry and retrieve its config.
        // (We don't call connect because that would try real SSH.)
        let session = SshSession::new("session-1".into(), config.clone());
        mgr.sessions.insert("session-1".into(), session);
        let s = mgr.get("session-1").unwrap();
        assert_eq!(s.id, "session-1");
        assert_eq!(s.config.host, "192.168.1.1");
    }

    #[test]
    fn test_get_nonexistent() {
        let mgr = SessionManager::new();
        assert!(mgr.get("ghost").is_none());
    }

    #[test]
    fn test_remove() {
        let mut mgr = SessionManager::new();
        let session = SshSession::new("s1".into(), make_config());
        mgr.sessions.insert("s1".into(), session);
        assert!(mgr.remove("s1").is_some());
        assert!(mgr.get("s1").is_none());
    }

    #[test]
    fn test_list() {
        let mut mgr = SessionManager::new();
        mgr.sessions
            .insert("a".into(), SshSession::new("a".into(), make_config()));
        mgr.sessions
            .insert("b".into(), SshSession::new("b".into(), make_config()));
        let list = mgr.list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"a".to_string()));
        assert!(list.contains(&"b".to_string()));
    }

    #[test]
    fn test_active_count() {
        let mut mgr = SessionManager::new();
        assert_eq!(mgr.active_count(), 0);
        mgr.sessions
            .insert("s1".into(), SshSession::new("s1".into(), make_config()));
        assert_eq!(mgr.active_count(), 1);
        mgr.remove("s1");
        assert_eq!(mgr.active_count(), 0);
    }
}
