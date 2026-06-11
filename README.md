<div align="center">

# HERMES

### Enterprise Remote Connection Manager

![Build Status](https://github.com/thichcode/rust-remotemanager/actions/workflows/build-windows.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-100%25-orange.svg)
![Platform](https://img.shields.io/badge/Platform-Windows-blue.svg)

**Secure. Fast. Professional.**

A enterprise-grade remote connection management solution built entirely in Rust.

[Download](#installation) · [Documentation](#features) · [Contributing](#contributing)

</div>

---

## Overview

Hermes is a professional remote connection manager designed for IT teams and system administrators who need secure, fast, and reliable access to remote systems.

Built with **100% Rust** using Tauri 2.x and Dioxus, Hermes delivers native performance with enterprise-level security.

### Key Highlights

- **Zero JavaScript** — Pure Rust frontend and backend
- **Military-grade Encryption** — AES-256-GCM with Argon2id key derivation
- **Native Performance** — No Electron, no overhead
- **Secure by Default** — Credentials encrypted at rest, vault-protected

---

## Features

### Connection Management

| Feature | Description |
|---------|-------------|
| **Multi-Protocol** | SSH, RDP, Serial, Telnet, VNC support |
| **Folder Organization** | Hierarchical folder structure for connection management |
| **Quick Search** | Instant search across all connections |
| **Favorites** | Pin frequently-accessed systems |
| **Tags & Notes** | Custom annotations for easy filtering |
| **Startup Commands** | Auto-execute scripts on connection |

### Terminal

| Feature | Description |
|---------|-------------|
| **Native Emulation** | Built-in terminal using termwiz (VT100/xterm compatible) |
| **Multi-Session** | Run multiple terminal sessions simultaneously |
| **Session Tabs** | Easy switching between active sessions |
| **Search** | Find text in terminal output |
| **Resize** | Dynamic terminal resizing |

### File Transfer

| Feature | Description |
|---------|-------------|
| **SFTP Browser** | Visual file browser with breadcrumb navigation |
| **Drag & Drop** | Upload files by dragging into the window |
| **Batch Operations** | Upload/download multiple files |
| **File Management** | Create, rename, delete files and folders |

### Security

| Feature | Description |
|---------|-------------|
| **Credential Vault** | Encrypted storage for passwords and SSH keys |
| **AES-256-GCM** | Industry-standard encryption |
| **Argon2id** | Memory-hard key derivation (OWASP compliant) |
| **Zeroization** | Secure memory cleanup for sensitive data |
| **Vault Lock** | Manual lock/unlock with master password |

### Port Forwarding

| Feature | Description |
|---------|-------------|
| **SSH Tunnels** | Local, remote, and SOCKS5 proxy support |
| **Tunnel Management** | Create, monitor, and stop tunnels |
| **Session Binding** | Tunnels linked to SSH sessions |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Hermes Architecture                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────┐    ┌─────────────────────────────┐   │
│  │   Dioxus UI     │    │      Rust Backend            │   │
│  │   (Frontend)    │◄──►│                              │   │
│  │                 │    │  ┌─────────────────────────┐ │   │
│  │  • Pages        │    │  │  SSH/SFTP Engine        │ │   │
│  │  • Components   │    │  │  (ssh2-rs)              │ │   │
│  │  • State        │    │  └─────────────────────────┘ │   │
│  │  • Router       │    │  ┌─────────────────────────┐ │   │
│  │                 │    │  │  Terminal Emulator       │ │   │
│  └─────────────────┘    │  │  (termwiz)              │ │   │
│                         │  └─────────────────────────┘ │   │
│                         │  ┌─────────────────────────┐ │   │
│                         │  │  Security Layer         │ │   │
│                         │  │  (AES-256 + Argon2id)   │ │   │
│                         │  └─────────────────────────┘ │   │
│                         │  ┌─────────────────────────┐ │   │
│                         │  │  SQLite Database         │ │   │
│                         │  │  (rusqlite)             │ │   │
│                         │  └─────────────────────────┘ │   │
│                         └─────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Tech Stack

| Layer | Technology |
|-------|------------|
| **Frontend** | Dioxus 0.6 (Rust) |
| **Backend** | Tauri 2.x (Rust) |
| **Terminal** | termwiz (WezTerm) |
| **SSH/SFTP** | ssh2-rs (libssh2) |
| **Encryption** | AES-256-GCM, Argon2id |
| **Database** | SQLite (rusqlite) |
| **Build** | Cargo, GitHub Actions |

---

## Installation

### Download

Download the latest release from [GitHub Releases](https://github.com/thichcode/rust-remotemanager/releases).

| Format | Description |
|--------|-------------|
| `HermesSetup.exe` | Windows installer (recommended) |
| `Hermes.msi` | Windows MSI package |
| `Hermes-portable.zip` | Portable version (no install) |

### Build from Source

**Prerequisites:**
- Rust 1.75+ (`rustup install stable`)
- Windows: Visual Studio Build Tools 2022

```bash
# Clone
git clone https://github.com/thichcode/rust-remotemanager.git
cd rust-remotemanager

# Build
cargo build --release

# Run
cargo run --release
```

---

## Usage

### Getting Started

1. **Launch Hermes** from Start Menu or portable folder
2. **Create a Connection** — Click "+" and enter SSH details
3. **Connect** — Double-click the connection card
4. **Manage Files** — Click the folder icon to open SFTP browser

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New connection |
| `Ctrl+F` | Search connections |
| `Ctrl+T` | New terminal tab |
| `Ctrl+W` | Close current tab |
| `Ctrl+Shift+C` | Copy selection |
| `Ctrl+Shift+V` | Paste |

### Credential Vault

1. Go to **Settings → Security**
2. Set a master password
3. Store credentials securely
4. Credentials are encrypted before storage

---

## Configuration

### Database Location

```
Windows: %APPDATA%\com.hermes.remote-manager\hermes.db
```

### Logs

```
Windows: %APPDATA%\com.hermes.remote-manager\logs\hermes.log.{date}
```

### Settings

All settings are stored in the database and can be managed via the UI.

---

## Development

### Project Structure

```
src-tauri/
├── src/
│   ├── commands/          # Tauri IPC handlers
│   ├── storage/           # SQLite database layer
│   ├── ssh/               # SSH/SFTP/Tunnel engine
│   ├── security/          # Encryption & vault
│   └── ui/                # Dioxus UI
│       ├── pages/         # Application pages
│       ├── components/    # Reusable components
│       ├── state.rs       # Global state management
│       └── services/      # Business logic
├── Cargo.toml
└── tauri.conf.json
```

### Available Scripts

```bash
# Development
cargo run                    # Run in debug mode
cargo run --release          # Run in release mode

# Build
cargo build                  # Build debug
cargo build --release        # Build release

# Test
cargo test                   # Run all tests
cargo test -- --nocapture    # Run with output

# Release
.\scripts\release.ps1 -Version "X.Y.Z"
```

### CI/CD

GitHub Actions automatically:
- Builds on push to `main`
- Creates release on tag push (`v*`)
- Generates changelog from commits
- Uploads installers and portable builds

---

## Security

### Encryption Standards

| Algorithm | Purpose |
|-----------|---------|
| AES-256-GCM | Data encryption |
| Argon2id | Key derivation (OWASP params) |
| HKDF-SHA256 | Sub-key derivation |
| SHA-256 | Hashing |

### Security Practices

- Credentials encrypted at rest
- Master password never stored (only derived key)
- Secure memory zeroization
- No network calls except SSH/SFTP
- Local-only operation (no telemetry)

---

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing`)
5. Open a Pull Request

### Code Style

- Follow Rust standard style (`cargo fmt`)
- Run clippy before commit (`cargo clippy`)
- Add tests for new features
- Update documentation as needed

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Support

- **Documentation:** [GitHub Wiki](https://github.com/thichcode/rust-remotemanager/wiki)
- **Issues:** [GitHub Issues](https://github.com/thichcode/rust-remotemanager/issues)
- **Discussions:** [GitHub Discussions](https://github.com/thichcode/rust-remotemanager/discussions)

---

<div align="center">

**Built with ❤️ in Rust**

[⬆ Back to Top](#hermes)

</div>
