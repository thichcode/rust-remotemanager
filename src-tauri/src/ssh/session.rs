use serde::{Deserialize, Serialize};
use ssh2::Channel;
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Configuration for establishing an SSH connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

impl SessionState {
    /// Convert to a simple string for the frontend.
    pub fn to_simple_string(&self) -> String {
        match self {
            SessionState::Disconnected => "disconnected".to_string(),
            SessionState::Connecting => "connecting".to_string(),
            SessionState::Connected { .. } => "connected".to_string(),
            SessionState::Error(_) => "error".to_string(),
        }
    }
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
    pub ssh_session: Arc<Mutex<Option<ssh2::Session>>>,
    pub cmd_tx: Option<Sender<ChannelCommand>>,
    pub thread_handle: Option<thread::JoinHandle<()>>,
    /// Buffer for terminal output emitted before the frontend subscribes.
    pub output_buffer: Arc<Mutex<VecDeque<String>>>,
    /// Once true, the I/O thread emits output directly instead of buffering.
    pub output_ready: Arc<AtomicBool>,
}

impl SshSession {
    fn new(id: String, config: SshConfig) -> Self {
        Self {
            id,
            config,
            state: Arc::new(Mutex::new(SessionState::Disconnected)),
            running: Arc::new(AtomicBool::new(false)),
            ssh_session: Arc::new(Mutex::new(None)),
            cmd_tx: None,
            thread_handle: None,
            output_buffer: Arc::new(Mutex::new(VecDeque::new())),
            output_ready: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Drop for SshSession {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(tx) = self.cmd_tx.take() {
            let _ = tx.send(ChannelCommand::Close);
        }
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Maximum number of simultaneous SSH sessions (prevents resource exhaustion).
pub const MAX_SESSIONS: usize = 20;

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
    ) -> Result<String, String> {
        // Enforce maximum session limit
        if self.sessions.len() >= MAX_SESSIONS {
            return Err(format!(
                "Maximum number of sessions ({}) reached", MAX_SESSIONS
            ));
        }
        let id = session_id.clone();

        let state = Arc::new(Mutex::new(SessionState::Connecting));
        let running = Arc::new(AtomicBool::new(true));
        let ssh_session_storage: Arc<Mutex<Option<ssh2::Session>>> = Arc::new(Mutex::new(None));
        let output_buffer: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
        let output_ready: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let (cmd_tx, cmd_rx) = mpsc::channel::<ChannelCommand>();

        let session = SshSession {
            id: id.clone(),
            config: config.clone(),
            state: state.clone(),
            running: running.clone(),
            cmd_tx: Some(cmd_tx),
            thread_handle: None,
            ssh_session: ssh_session_storage.clone(),
            output_buffer: output_buffer.clone(),
            output_ready: output_ready.clone(),
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
                    ssh_session_storage,
                    output_buffer,
                    output_ready,
                );
            })
            .expect("failed to spawn SSH I/O thread");

        // Update the session with the thread handle.
        if let Some(s) = self.sessions.get_mut(&id) {
            s.thread_handle = Some(handle);
        }

        Ok(id)
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

    /// Set output_ready to true and return all buffered terminal output.
    /// Called by the frontend after it registers its terminal:output listener,
    /// ensuring the I/O thread emits directly from this point onward.
    pub fn flush_output(&self, id: &str) -> Option<Vec<String>> {
        let session = self.sessions.get(id)?;
        tracing::info!(
            "[{}] flush_output called, buffered {} items before switching to direct emit",
            id,
            session.output_buffer.lock().unwrap().len()
        );
        session.output_ready.store(true, Ordering::SeqCst);
        let mut buf = session.output_buffer.lock().unwrap();
        let result: Vec<String> = buf.drain(..).collect();
        tracing::info!("[{}] flush_output drained {} items, output_ready=true", id, result.len());
        Some(result)
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

    /// Get a clone of the underlying ssh2::Session for SFTP operations.
    pub fn get_ssh_session(&self, id: &str) -> Option<ssh2::Session> {
        self.sessions.get(id).and_then(|s| {
            s.ssh_session
                .lock()
                .ok()
                .and_then(|guard| guard.clone())
        })
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
    ssh_session_storage: Arc<Mutex<Option<ssh2::Session>>>,
    output_buffer: Arc<Mutex<VecDeque<String>>>,
    output_ready: Arc<AtomicBool>,
) {
    let id = session_id.to_string();
    tracing::info!("[{}] SSH I/O thread started, output_ready=false", id);
    let result = establish_connection(config);

    let mut channel = match result {
        Ok((session, ch)) => {
            // Set session non-blocking so channel.read() returns WouldBlock
            session.set_blocking(false);
            // Store for SFTP operations
            *ssh_session_storage.lock().unwrap() = Some(session);
            *state.lock().unwrap() = SessionState::Connected {
                cols: 80,
                rows: 24,
            };
            tracing::info!("[{}] SSH connected, emitting terminal:connected-{}", id, id);
            let _ = app_handle.emit(
                &format!("terminal:connected-{}", id),
                serde_json::json!({
                    "id": id,
                    "cols": 80,
                    "rows": 24,
                }),
            );
            tracing::info!("[{}] terminal:connected-{} emitted successfully", id, id);
            ch
        }
        Err(e) => {
            *state.lock().unwrap() = SessionState::Error(e.clone());
            let _ = app_handle.emit(
                &format!("terminal:error-{}", id),
                serde_json::json!({
                    "id": id,
                    "error": e,
                }),
            );
            return;
        }
    };

    let mut buf = [0u8; 8192];
    let mut line_buf = String::new();
    let mut keepalive_counter: u32 = 0;

    // ── Main I/O loop ──────────────────────────────────────────────
    // Optimized: use recv_timeout as the primary blocking point (~33 iterations/sec idle)
    // instead of busy-waiting with sleep(1ms) (~1000 iterations/sec).
    loop {
        // 1. Block on command channel with timeout (30ms).
        //    This is the primary sleep point — when idle, we block here
        //    instead of busy-waiting, reducing CPU from ~5-10% to ~0% per session.
        match cmd_rx.recv_timeout(Duration::from_millis(30)) {
            Ok(ChannelCommand::Input(data)) => {
                if let Err(err) = channel.write_all(&data) {
                    tracing::error!("[{}] write error: {}", id, err);
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
            Ok(ChannelCommand::Close) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // Graceful shutdown.
                let _ = channel.send_eof();
                let _ = channel.wait_close();
                *state.lock().unwrap() = SessionState::Disconnected;
                return;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Timeout — check for pending output and keepalive
            }
        }

        if !running.load(Ordering::SeqCst) {
            break;
        }

        // 2. Process any remaining commands that arrived during recv (non-blocking drain).
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

        // 3. Read stdout from the channel (non-blocking).
        loop {
            match channel.read(&mut buf) {
                Ok(0) => {
                    // EOF from remote side — shell likely exited.
                    tracing::info!("[{}] EOF on stdout", id);
                    flush_line_buf(&mut line_buf, &id, &output_buffer, &output_ready, &app_handle);
                    let _ = app_handle.emit(
                        &format!("terminal:exit-{}", id),
                        serde_json::json!({ "id": id }),
                    );
                    *state.lock().unwrap() = SessionState::Disconnected;
                    return;
                }
                Ok(n) => {
                    line_buf.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if line_buf.len() >= 4096 {
                        let data = std::mem::take(&mut line_buf);
                        emit_or_buffer(data, &id, &output_buffer, &output_ready, &app_handle);
                    }
                    continue;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No more data right now.
                }
                Err(e) => {
                    tracing::error!("[{}] read error: {}", id, e);
                    let _ = app_handle.emit(
                        &format!("terminal:error-{}", id),
                        serde_json::json!({ "id": id, "error": e.to_string() }),
                    );
                    *state.lock().unwrap() = SessionState::Error(e.to_string());
                    return;
                }
            }
            break;
        }

        // Drain remaining line_buf
        flush_line_buf(&mut line_buf, &id, &output_buffer, &output_ready, &app_handle);

        // 4. Read stderr from the channel (non-blocking).
        loop {
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

        // 5. Periodic keepalive send (every ~1s = ~33 iterations at 30ms timeout)
        keepalive_counter += 1;
        if keepalive_counter >= 33 {
            keepalive_counter = 0;
            if let Some(ref session) = *ssh_session_storage.lock().unwrap() {
                let _ = session.keepalive_send();
            }
        }
    }

    // Cleanup when `running` was set to false externally.
    *state.lock().unwrap() = SessionState::Disconnected;
    let _ = channel.send_eof();
    let _ = channel.wait_close();
}

/// Helper: flush the line buffer to output (emit or buffer).
fn flush_line_buf(
    line_buf: &mut String,
    id: &str,
    output_buffer: &Arc<Mutex<VecDeque<String>>>,
    output_ready: &Arc<AtomicBool>,
    app_handle: &AppHandle,
) {
    if line_buf.is_empty() {
        return;
    }
    let data = std::mem::take(line_buf);
    emit_or_buffer(data, id, output_buffer, output_ready, app_handle);
}

/// Helper: emit data via Tauri event if output_ready, otherwise buffer it.
fn emit_or_buffer(
    data: String,
    id: &str,
    output_buffer: &Arc<Mutex<VecDeque<String>>>,
    output_ready: &Arc<AtomicBool>,
    app_handle: &AppHandle,
) {
    if output_ready.load(Ordering::SeqCst) {
        let _ = app_handle.emit(
            &format!("terminal:output-{}", id),
            serde_json::json!({ "id": id, "data": data }),
        );
    } else {
        output_buffer.lock().unwrap().push_back(data);
    }
}

/// Perform the full SSH connection sequence:
/// TCP connect → SSH handshake → authenticate → open channel → PTY → shell.
/// Returns both the `Session` and `Channel` so the session can be cloned for SFTP.
fn establish_connection(config: &SshConfig) -> Result<(ssh2::Session, Channel), String> {
    // 1. TCP connect with 15s timeout
    let addr = format!("{}:{}", config.host, config.port);
    let addrs: Vec<std::net::SocketAddr> = addr
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed for {}: {}", addr, e))?
        .collect();
    let socket_addr = addrs
        .first()
        .ok_or_else(|| format!("No addresses resolved for {}", addr))?;
    let tcp = TcpStream::connect_timeout(socket_addr, Duration::from_secs(15))
        .map_err(|e| format!("TCP connection to {} failed: {}", addr, e))?;

    // 2. Create SSH session and handshake
    let mut session =
        ssh2::Session::new().map_err(|e| format!("Failed to create SSH session: {}", e))?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|e| format!("SSH handshake with {} failed: {}", config.host, e))?;

    // 3. Enable SSH keepalive if configured
    if let Some(interval) = config.keepalive_interval {
        if interval > 0 {
            session.set_keepalive(true, interval);
        }
    }

    // 4. Authenticate
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

    // 5. Open channel, request PTY, start shell
    let mut channel = session
        .channel_session()
        .map_err(|e| format!("Failed to open SSH channel: {}", e))?;

    channel
        .request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
        .map_err(|e| format!("Failed to request PTY: {}", e))?;

    channel
        .shell()
        .map_err(|e| format!("Failed to start shell: {}", e))?;

    Ok((session, channel))
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
