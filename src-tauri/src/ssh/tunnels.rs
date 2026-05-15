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
    pub fn add_local(
        &self,
        session: &ssh2::Session,
        config: TunnelConfig,
    ) -> AppResult<TunnelConfig> {
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
                while r.load(Ordering::SeqCst) {
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
                            thread::sleep(Duration::from_millis(100));
                        }
                        Err(e) => {
                            if r.load(Ordering::SeqCst) {
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
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        let sess = session.clone();
        let local_addr = format!("{}:{}", config.local_host, config.local_port);
        let tunnel_name = config.name.clone();

        // Ask the remote server to listen. `host==None` means listen on all
        // interfaces (or the SSH server's configured gateway ports limit).
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
                while r.load(Ordering::SeqCst) {
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
                            if r.load(Ordering::SeqCst) {
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
                // Listener is dropped here, which cancels the remote forward.
                tracing::info!("Remote tunnel '{}' stopped", tunnel_name);
            })
            .map_err(|e| crate::error::AppError::Io(e.to_string()))?;

        // Update config with the actual bound port
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
    /// Incoming connections are parsed as SOCKS5 requests and forwarded
    /// to the target host:port through the SSH session.
    pub fn add_dynamic(
        &self,
        session: &ssh2::Session,
        config: TunnelConfig,
    ) -> AppResult<TunnelConfig> {
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
                while r.load(Ordering::SeqCst) {
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
                            thread::sleep(Duration::from_millis(100));
                        }
                        Err(e) => {
                            if r.load(Ordering::SeqCst) {
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

    /// Stop and remove a tunnel by ID. Sets the stop flag and joins the
    /// tunnel thread.
    pub fn remove(&self, id: &str) {
        let mut tunnels = match self.tunnels.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if let Some(mut tunnel) = tunnels.remove(id) {
            tunnel.running.store(false, Ordering::SeqCst);
            if let Some(handle) = tunnel.thread_handle.take() {
                let _ = handle.join();
            }
            tracing::info!("Tunnel '{}' removed", id);
        }
    }

    /// List all active tunnel configs.
    pub fn list(&self) -> Vec<TunnelConfig> {
        self.tunnels
            .lock()
            .map(|g| g.values().map(|t| t.config.clone()).collect())
            .unwrap_or_default()
    }

    /// Stop all tunnels.
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

fn forward_connection(local: TcpStream, remote: Channel, name: &str) {
    // Set both to non-blocking for single-threaded bidirectional forwarding
    let mut local = local;
    let mut remote = remote;
    let _ = local.set_nonblocking(true);
    let mut buf = [0u8; 32768];
    loop {
        // Direction: TCP → SSH channel
        match local.read(&mut buf) {
            Ok(0) => break, // TCP closed
            Ok(n) => {
                if remote.write_all(&buf[..n]).is_err() || remote.flush().is_err() {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => break,
        }
        // Direction: SSH channel → TCP
        match remote.read(&mut buf) {
            Ok(0) => break, // SSH channel EOF
            Ok(n) => {
                if local.write_all(&buf[..n]).is_err() || local.flush().is_err() {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => break,
        }
        // Small sleep to prevent busy-waiting
        thread::sleep(Duration::from_millis(10));
    }
    let _ = local.shutdown(std::net::Shutdown::Both);
    tracing::debug!("Tunnel '{}' connection closed", name);
}

// ─── SOCKS5 Handler ──────────────────────────────────────────────────────────

/// Handle a single SOCKS5 client connection.
///
/// Implements the minimal SOCKS5 protocol (RFC 1928):
/// 1. Read client greeting (auth methods)
/// 2. Reply with "no authentication"
/// 3. Read client request (CONNECT to target host:port)
/// 4. Open SSH channel to target
/// 5. Reply with success
/// 6. Forward data bidirectionally
fn handle_socks5(mut stream: TcpStream, session: &ssh2::Session) -> Result<(), String> {
    

    // Set a reasonable read timeout
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("set_read_timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("set_write_timeout: {}", e))?;

    // Disable Nagle for lower latency on the tunnel
    stream
        .set_nodelay(true)
        .map_err(|e| format!("set_nodelay: {}", e))?;

    // ── Step 1: Read greeting ──────────────────────────────────────────
    // Client sends: version(1) + nmethods(1) + methods(nmethods)
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

    // ── Step 2: Reply with "no authentication required" (0x00) ─────────
    // If 0x00 is not in the client's method list, reply with rejection.
    let no_auth = methods.contains(&0x00);
    if !no_auth {
        // Reject: version(5) + auth_method(0xFF = no acceptable methods)
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

    // ── Step 3: Read request ───────────────────────────────────────────
    // Client sends:
    //   ver(1) + cmd(1) + rsv(1) + atyp(1) + dst.addr(var) + dst.port(2)
    //
    // atyp: 1=IPv4, 3=domain, 4=IPv6
    // cmd:  1=CONNECT
    //
    // We only support CONNECT (cmd=1).

    let mut header = [0u8; 4];
    stream
        .read_exact(&mut header)
        .map_err(|e| format!("read request header: {}", e))?;

    let _ver = header[0];
    let cmd = header[1];
    let _rsv = header[2];
    let atyp = header[3];

    if cmd != 1 {
        return Err(format!("unsupported SOCKS5 command: {} (only CONNECT=1)", cmd));
    }

    let target_host = match atyp {
        1 => {
            // IPv4
            let mut addr = [0u8; 4];
            stream
                .read_exact(&mut addr)
                .map_err(|e| format!("read IPv4: {}", e))?;
            format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3])
        }
        3 => {
            // Domain name
            let mut len_byte = [0u8; 1];
            stream
                .read_exact(&mut len_byte)
                .map_err(|e| format!("read domain len: {}", e))?;
            let domain_len = len_byte[0] as usize;
            let mut domain = vec![0u8; domain_len];
            stream
                .read_exact(&mut domain)
                .map_err(|e| format!("read domain: {}", e))?;
            String::from_utf8_lossy(&domain).to_string()
        }
        4 => {
            // IPv6 (not commonly supported, but we parse it)
            let mut addr = [0u8; 16];
            stream
                .read_exact(&mut addr)
                .map_err(|e| format!("read IPv6: {}", e))?;
            // Format as colon-separated hex groups
            let groups: Vec<String> = addr.chunks(2).map(|c| format!("{:02x}{:02x}", c[0], c[1])).collect();
            groups.join(":")
        }
        _ => return Err(format!("unsupported address type: {}", atyp)),
    };

    let mut port_bytes = [0u8; 2];
    stream
        .read_exact(&mut port_bytes)
        .map_err(|e| format!("read port: {}", e))?;
    let target_port = u16::from_be_bytes(port_bytes);

    tracing::debug!("SOCKS5 CONNECT {}:{}", target_host, target_port);

    // ── Step 4: Open SSH direct TCP/IP channel ─────────────────────────
    let channel = session
        .channel_direct_tcpip(&target_host, target_port, None)
        .map_err(|e| format!("channel_direct_tcpip: {}", e))?;

    // ── Step 5: Send SOCKS5 reply ──────────────────────────────────────
    // Reply format: ver(1) + rep(1) + rsv(1) + atyp(1) + bnd.addr(var) + bnd.port(2)
    // rep=0 means success
    let reply = [
        0x05, // version
        0x00, // success
        0x00, // reserved
        0x01, // IPv4
        0x00, 0x00, 0x00, 0x00, // bind address (0.0.0.0)
        0x00, 0x00, // bind port (0)
    ];
    stream
        .write_all(&reply)
        .map_err(|e| format!("write reply: {}", e))?;

    // ── Step 6: Forward data bidirectionally ───────────────────────────
    forward_connection(stream, channel, &format!("socks5-{}:{}", target_host, target_port));

    Ok(())
}
