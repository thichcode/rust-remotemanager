use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use ssh2::Channel;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Maximum number of simultaneous tunnels (prevents resource exhaustion).
const MAX_TUNNELS: usize = 50;

// ─── Data Types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TunnelType {
    Local,
    Remote,
    Dynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub id: String,
    pub session_id: String,
    pub tunnel_type: TunnelType,
    pub name: String,
    pub local_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub active: bool,
}

// ─── Tunnel Instance ─────────────────────────────────────────────────────────

pub struct Tunnel {
    pub config: TunnelConfig,
    pub running: Arc<AtomicBool>,
    pub thread_handle: Option<thread::JoinHandle<()>>,
}

// ─── Tunnel Manager ──────────────────────────────────────────────────────────

pub struct TunnelManager {
    tunnels: Arc<Mutex<HashMap<String, Tunnel>>>,
}

impl Default for TunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            tunnels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start a local port forward: listen on local_host:local_port, forward TCP
    /// connections through the SSH session to remote_host:remote_port.
    /// Uses non-blocking accept with 50ms poll interval (was 100µs) — reduces
    /// CPU from ~10,000 iterations/s to ~20 iterations/s when idle.
    pub fn add_local(
        &self,
        session: &ssh2::Session,
        config: TunnelConfig,
    ) -> AppResult<TunnelConfig> {
        // Enforce maximum tunnel limit to prevent resource exhaustion
        let current_count = self.tunnels.lock().map_err(|e| crate::error::AppError::Ssh(format!("Lock error: {}", e)))?.len();
        if current_count >= MAX_TUNNELS {
            return Err(crate::error::AppError::Validation(format!(
                "Maximum number of tunnels ({}) reached", MAX_TUNNELS
            )));
        }
        let bind_addr = format!("{}:{}", config.local_host, config.local_port);
        let listener = TcpListener::bind(&bind_addr)?;
        listener
            .set_nonblocking(true)
            .map_err(|e| crate::error::AppError::Io(e.to_string()))?;

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        let sess = session.clone();
        let remote_host = config.remote_host.clone();
        let remote_port = config.remote_port;
        let tunnel_name = config.name.clone();

        let handle = thread::Builder::new()
            .name(format!("tunnel-local-{}", &config.id[..8.min(config.id.len())]))
            .spawn(move || {
                tracing::info!(
                    "Local tunnel '{}' listening on {} -> {}:{}",
                    tunnel_name,
                    bind_addr,
                    remote_host,
                    remote_port
                );
                while r.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((tcp, peer_addr)) => {
                            let sess = sess.clone();
                            let remote_host = remote_host.clone();
                            let tunnel_name = tunnel_name.clone();
                            thread::spawn(move || {
                                tracing::debug!(
                                    "Tunnel '{}': accepted connection from {}",
                                    tunnel_name,
                                    peer_addr
                                );
                                match sess.channel_direct_tcpip(&remote_host, remote_port, None) {
                                    Ok(channel) => {
                                        forward_connection(tcp, channel, &tunnel_name);
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Tunnel '{}': failed to open direct channel: {}",
                                            tunnel_name,
                                            e
                                        );
                                    }
                                }
                            });
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // 50ms sleep instead of 100µs — 20 iterations/s idle instead of 10,000
                            thread::sleep(Duration::from_millis(50));
                        }
                        Err(e) => {
                            if r.load(Ordering::Relaxed) {
                                tracing::error!(
                                    "Tunnel '{}' listener error: {}",
                                    tunnel_name,
                                    e
                                );
                            }
                            break;
                        }
                    }
                }
                tracing::info!("Local tunnel '{}' stopped", tunnel_name);
            })
            .map_err(|e| crate::error::AppError::Io(e.to_string()))?;

        let tunnel = Tunnel {
            config: config.clone(),
            running,
            thread_handle: Some(handle),
        };

        self.tunnels
            .lock()
            .map_err(|e| crate::error::AppError::Ssh(format!("Lock error: {}", e)))?
            .insert(config.id.clone(), tunnel);

        Ok(config)
    }

    /// Start a remote port forward: ask the SSH server to listen on
    /// remote_host:remote_port and forward incoming connections to
    /// local_host:local_port via the SSH session.
    pub fn add_remote(
        &self,
        session: &ssh2::Session,
        config: TunnelConfig,
    ) -> AppResult<TunnelConfig> {
        // Enforce maximum tunnel limit
        {
            let tunnels = self.tunnels.lock().map_err(|e| crate::error::AppError::Ssh(format!("Lock error: {}", e)))?;
            if tunnels.len() >= MAX_TUNNELS {
                return Err(crate::error::AppError::Validation(format!(
                    "Maximum number of tunnels ({}) reached", MAX_TUNNELS
                )));
            }
        }
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        let sess = session.clone();
        let local_addr = format!("{}:{}", config.local_host, config.local_port);
        let tunnel_name = config.name.clone();

        let remote_host_opt = if config.remote_host.is_empty() || config.remote_host == "0.0.0.0"
        {
            None
        } else {
            Some(config.remote_host.as_str())
        };

        let (mut listener, bound_port) = sess
            .channel_forward_listen(config.remote_port, remote_host_opt, Some(10))
            .map_err(|e| {
                crate::error::AppError::Ssh(format!(
                    "Failed to listen on remote port {}: {}",
                    config.remote_port, e
                ))
            })?;

        let effective_port = if config.remote_port == 0 {
            bound_port
        } else {
            config.remote_port
        };

        tracing::info!(
            "Remote tunnel '{}' listening on remote port {} -> {}",
            tunnel_name,
            effective_port,
            local_addr,
        );

        let handle = thread::Builder::new()
            .name(format!("tunnel-remote-{}", &config.id[..8.min(config.id.len())]))
            .spawn(move || {
                while r.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok(channel) => {
                            let local_addr = local_addr.clone();
                            let tunnel_name = tunnel_name.clone();
                            thread::spawn(move || {
                                tracing::debug!(
                                    "Tunnel '{}': accepted remote forwarding",
                                    tunnel_name
                                );
                                match TcpStream::connect(&local_addr) {
                                    Ok(tcp) => {
                                        forward_connection(tcp, channel, &tunnel_name);
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Tunnel '{}': failed to connect to {}: {}",
                                            tunnel_name,
                                            local_addr,
                                            e
                                        );
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            if r.load(Ordering::Relaxed) {
                                tracing::error!(
                                    "Tunnel '{}' accept error: {}",
                                    tunnel_name,
                                    e
                                );
                            }
                            break;
                        }
                    }
                }
                tracing::info!("Remote tunnel '{}' stopped", tunnel_name);
            })
            .map_err(|e| crate::error::AppError::Io(e.to_string()))?;

        let mut config = config;
        config.remote_port = effective_port;
        config.active = true;

        let tunnel = Tunnel {
            config: config.clone(),
            running,
            thread_handle: Some(handle),
        };

        self.tunnels
            .lock()
            .map_err(|e| crate::error::AppError::Ssh(format!("Lock error: {}", e)))?
            .insert(config.id.clone(), tunnel);

        Ok(config)
    }

    /// Start a SOCKS5 dynamic proxy on local_host:local_port.
    pub fn add_dynamic(
        &self,
        session: &ssh2::Session,
        config: TunnelConfig,
    ) -> AppResult<TunnelConfig> {
        // Enforce maximum tunnel limit
        {
            let tunnels = self.tunnels.lock().map_err(|e| crate::error::AppError::Ssh(format!("Lock error: {}", e)))?;
            if tunnels.len() >= MAX_TUNNELS {
                return Err(crate::error::AppError::Validation(format!(
                    "Maximum number of tunnels ({}) reached", MAX_TUNNELS
                )));
            }
        }
        let bind_addr = format!("{}:{}", config.local_host, config.local_port);
        let listener = TcpListener::bind(&bind_addr)?;
        listener
            .set_nonblocking(true)
            .map_err(|e| crate::error::AppError::Io(e.to_string()))?;

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        let sess = session.clone();
        let tunnel_name = config.name.clone();

        let handle = thread::Builder::new()
            .name(format!("tunnel-dyn-{}", &config.id[..8.min(config.id.len())]))
            .spawn(move || {
                tracing::info!(
                    "Dynamic SOCKS5 tunnel '{}' listening on {}",
                    tunnel_name,
                    bind_addr
                );
                while r.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((tcp, peer_addr)) => {
                            let sess = sess.clone();
                            let tunnel_name = tunnel_name.clone();
                            thread::spawn(move || {
                                tracing::debug!(
                                    "SOCKS5 '{}': connection from {}",
                                    tunnel_name,
                                    peer_addr
                                );
                                if let Err(e) = handle_socks5(tcp, &sess) {
                                    tracing::error!(
                                        "SOCKS5 '{}' error: {}",
                                        tunnel_name,
                                        e
                                    );
                                }
                            });
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // 50ms poll interval — reduced from 100µs for ~500× CPU savings
                            thread::sleep(Duration::from_millis(50));
                        }
                        Err(e) => {
                            if r.load(Ordering::Relaxed) {
                                tracing::error!(
                                    "Dynamic tunnel '{}' listener error: {}",
                                    tunnel_name,
                                    e
                                );
                            }
                            break;
                        }
                    }
                }
                tracing::info!("Dynamic tunnel '{}' stopped", tunnel_name);
            })
            .map_err(|e| crate::error::AppError::Io(e.to_string()))?;

        let tunnel = Tunnel {
            config: config.clone(),
            running,
            thread_handle: Some(handle),
        };

        self.tunnels
            .lock()
            .map_err(|e| crate::error::AppError::Ssh(format!("Lock error: {}", e)))?
            .insert(config.id.clone(), tunnel);

        Ok(config)
    }

    pub fn remove(&self, id: &str) {
        let mut tunnels = match self.tunnels.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if let Some(mut tunnel) = tunnels.remove(id) {
            tunnel.running.store(false, Ordering::Relaxed);
            if let Some(handle) = tunnel.thread_handle.take() {
                let _ = handle.join();
            }
            tracing::info!("Tunnel '{}' removed", id);
        }
    }

    pub fn list(&self) -> Vec<TunnelConfig> {
        self.tunnels
            .lock()
            .map(|g| g.values().map(|t| t.config.clone()).collect())
            .unwrap_or_default()
    }

    pub fn stop_all(&self) {
        let ids: Vec<String> = self
            .tunnels
            .lock()
            .map(|g| g.keys().cloned().collect())
            .unwrap_or_default();
        for id in ids {
            self.remove(&id);
        }
    }
}

// ─── Connection Forwarding ───────────────────────────────────────────────────

/// Bidirectional forwarding between TCP and SSH channel.
/// Uses blocking I/O with a short sleep to prevent busy-waiting.
fn forward_connection(local: TcpStream, remote: Channel, name: &str) {
    let mut local = local;
    let mut remote = remote;
    local.set_nonblocking(true).ok();
    let mut buf = [0u8; 32768];
    loop {
        match local.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if remote.write_all(&buf[..n]).is_err() || remote.flush().is_err() {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => break,
        }
        match remote.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if local.write_all(&buf[..n]).is_err() || local.flush().is_err() {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => break,
        }
        // 1ms sleep instead of 10ms for lower latency
        thread::sleep(Duration::from_millis(1));
    }
    let _ = local.shutdown(std::net::Shutdown::Both);
    tracing::debug!("Tunnel '{}' connection closed", name);
}

// ─── SOCKS5 Handler ──────────────────────────────────────────────────────────

fn handle_socks5(mut stream: TcpStream, session: &ssh2::Session) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("set_read_timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("set_write_timeout: {}", e))?;
    stream
        .set_nodelay(true)
        .map_err(|e| format!("set_nodelay: {}", e))?;

    // Step 1: Read greeting
    let mut greeting = [0u8; 2];
    stream
        .read_exact(&mut greeting)
        .map_err(|e| format!("read greeting: {}", e))?;
    let ver = greeting[0];
    let nmethods = greeting[1] as usize;
    if ver != 5 {
        return Err(format!("unsupported SOCKS version: {}", ver));
    }
    let mut methods = vec![0u8; nmethods];
    if nmethods > 0 {
        stream
            .read_exact(&mut methods)
            .map_err(|e| format!("read methods: {}", e))?;
    }

    // Step 2: Reply with "no authentication required" (0x00)
    let no_auth = methods.contains(&0x00);
    if !no_auth {
        let reject = [0x05, 0xFF];
        stream
            .write_all(&reject)
            .map_err(|e| format!("write reject: {}", e))?;
        return Err("client does not support no-auth".to_string());
    }
    let auth_reply = [0x05, 0x00];
    stream
        .write_all(&auth_reply)
        .map_err(|e| format!("write auth reply: {}", e))?;

    // Step 3: Read request
    let mut header = [0u8; 4];
    stream
        .read_exact(&mut header)
        .map_err(|e| format!("read request header: {}", e))?;
    let cmd = header[1];
    let atyp = header[3];
    if cmd != 1 {
        return Err(format!("unsupported SOCKS5 command: {} (only CONNECT=1)", cmd));
    }
    let target_host = match atyp {
        1 => {
            let mut addr = [0u8; 4];
            stream.read_exact(&mut addr).map_err(|e| format!("read IPv4: {}", e))?;
            format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3])
        }
        3 => {
            let mut len_byte = [0u8; 1];
            stream.read_exact(&mut len_byte).map_err(|e| format!("read domain len: {}", e))?;
            let domain_len = len_byte[0] as usize;
            let mut domain = vec![0u8; domain_len];
            stream.read_exact(&mut domain).map_err(|e| format!("read domain: {}", e))?;
            String::from_utf8_lossy(&domain).to_string()
        }
        4 => {
            let mut addr = [0u8; 16];
            stream.read_exact(&mut addr).map_err(|e| format!("read IPv6: {}", e))?;
            let groups: Vec<String> = addr.chunks(2).map(|c| format!("{:02x}{:02x}", c[0], c[1])).collect();
            groups.join(":")
        }
        _ => return Err(format!("unsupported address type: {}", atyp)),
    };
    let mut port_bytes = [0u8; 2];
    stream.read_exact(&mut port_bytes).map_err(|e| format!("read port: {}", e))?;
    let target_port = u16::from_be_bytes(port_bytes);

    tracing::debug!("SOCKS5 CONNECT {}:{}", target_host, target_port);

    // Step 4: Open SSH channel
    let channel = session
        .channel_direct_tcpip(&target_host, target_port, None)
        .map_err(|e| format!("channel_direct_tcpip: {}", e))?;

    // Step 5: Send SOCKS5 reply
    let reply: [u8; 10] = [0x05, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    stream.write_all(&reply).map_err(|e| format!("write reply: {}", e))?;

    // Step 6: Forward data
    forward_connection(stream, channel, &format!("socks5-{}:{}", target_host, target_port));
    Ok(())
}