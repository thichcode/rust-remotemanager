# Hermes Remote Manager

![Build Status](https://github.com/thichcode/rust-remotemanager/actions/workflows/build-windows.yml/badge.svg)

**Hermes Remote Manager** — A cross-platform desktop application for managing remote connections (SSH, RDP, Serial), with built-in terminal, SFTP file browser, port forwarding, and credential vault.

Built with **Tauri 2.x** (Rust backend) + **React 18 / TypeScript** (frontend).

---

## 🚀 Features

| Feature | Description |
|---------|-------------|
| **Connection Management** | Create, edit, delete, search, and organize connections into folders |
| **SSH Terminal** | Native terminal emulator via xterm.js with SSH2 backend |
| **SFTP File Browser** | Browse, download, upload, create, rename, delete remote files |
| **Port Forwarding / Tunnels** | Create and manage SSH tunnels (local forwarding) |
| **Credential Vault** | Store encrypted credentials (password + private key) securely |
| **Favorites** | Pin frequently-used connections for quick access |
| **Tags & Notes** | Annotate connections for easy filtering |
| **Startup Commands** | Auto-run commands on session connect |
| **Portable Mode** | Run directly from a folder — no installer required |

---

## 📦 Prerequisites

- **Rust** 1.75+ (`rustup install stable`)
- **Node.js** 20+ (`nvm use 20`)
- **npm** 9+ (comes with Node.js)
- **Tauri CLI** (auto-installed during build)
- Windows: Visual Studio Build Tools 2022 (for MSVC target)

---

## 🔧 Build & Run

```bash
# 1. Clone the repo
git clone https://github.com/thichcode/rust-remotemanager.git
cd rust-remotemanager

# 2. Install frontend dependencies
npm install

# 3. Build frontend (Vite + React)
npm run build

# 4. Build & run the Tauri app
cargo tauri dev

# Or release build:
cargo tauri build
```

---

## 📋 Project Structure

```
├── src-tauri/
│   ├── src/
│   │   ├── commands/          # Tauri IPC command handlers
│   │   │   ├── connections.rs # CRUD for remote connections
│   │   │   ├── folders.rs     # Folder hierarchy management
│   │   │   ├── credentials.rs # Encrypted credential storage
│   │   │   ├── terminal.rs    # SSH session & terminal
│   │   │   ├── sftp.rs        # SFTP file operations
│   │   │   ├── tunnels.rs     # SSH tunnel / port forwarding
│   │   │   ├── vault.rs       # Vault lock/unlock
│   │   │   └── settings.rs    # App settings
│   │   ├── storage/           # SQLite persistence (rusqlite)
│   │   │   ├── database.rs    # DB connection & migrations
│   │   │   ├── models.rs      # Data models (Connection, Folder, etc.)
│   │   │   └── repositories/  # Repository pattern (DAO)
│   │   ├── security/          # Encryption (aes-gcm, argon2)
│   │   └── main.rs            # Tauri app entry point
│   └── Cargo.toml
├── src/
│   ├── services/
│   │   └── ipc.ts             # Tauri invoke wrappers (frontend → Rust)
│   ├── components/            # React UI components
│   ├── stores/                # Zustand global state
│   ├── pages/                 # Route pages
│   ├── types.ts               # TypeScript type definitions
│   └── App.tsx                # React Router entry
├── scripts/
│   ├── portable-zip.ps1       # PowerShell: create portable zip (Windows)
│   └── portable-zip.sh        # Bash: create portable zip (Linux/macOS)
├── .github/workflows/
│   └── build-windows.yml      # GitHub Actions CI (Windows)
├── vite.config.ts             # Vite config
├── tailwind.config.js         # Tailwind CSS config
└── package.json
```

---

## 📦 Builds

| Format | Output | Notes |
|--------|--------|-------|
| **MSI** | `src-tauri/target/release/bundle/msi/*.msi` | Windows installer |
| **NSIS** | `src-tauri/target/release/bundle/nsis/*.exe` | Windows setup exe |
| **Portable ZIP** | `artifacts/hermes-remote-manager-v*.zip` | No install needed — just unzip & run |

---

## 🤖 CI/CD (GitHub Actions)

The `.github/workflows/build-windows.yml` workflow:

1. **Triggers**: Push to `main`, tags `v*`, or manual dispatch
2. **Steps**:
   - Checkout → Setup Rust + Node.js → Cache deps
   - Build frontend (`npm run build`) → Build Tauri app
   - Create portable zip via `scripts/portable-zip.ps1`
   - Upload MSI, NSIS, and ZIP as build artifacts
   - Auto-create GitHub Release on tag push with all binaries

---

## 🗄️ Database

- **Engine**: SQLite via `rusqlite`
- **Location**: Platform app data directory (`%APPDATA%` on Windows)
- **Schema**: Auto-migrated on first launch (`src-tauri/src/storage/migrations/`)
- **Tables**: `connections`, `folders`, `credentials`, `settings`, `snippets`

---

## 🔐 Security

- Credentials encrypted with **AES-256-GCM** before storage
- Key derived via **Argon2id** from vault password
- Vault stays locked until user enters password
- Private keys encrypted at rest; decrypted only in memory during SSH connect

---

## 📖 API (Tauri Commands)

All IPC commands are defined in `src-tauri/src/commands/` and exposed to the frontend via `@tauri-apps/api`:

| Command | Module | Purpose |
|---------|--------|---------|
| `list_connections` | connections | Get all connections |
| `create_connection` | connections | Add new connection |
| `update_connection` | connections | Edit existing connection |
| `delete_connection` | connections | Remove connection |
| `search_connections` | connections | Fuzzy search |
| `get_connection` | connections | Get by ID |
| `get_favorites` | connections | Get favorited connections |
| `list_folders` | folders | Get folder tree |
| `create_folder` | folders | Add folder |
| `update_folder` | folders | Edit folder |
| `delete_folder` | folders | Remove folder |
| `reorder_folders` | folders | Reorder via drag |
| `list_credentials` | credentials | Get encrypted credentials |
| `save_credential` | credentials | Store credential |
| `delete_credential` | credentials | Remove credential |
| `connect_ssh` | terminal | Open SSH session |
| `disconnect_session` | terminal | Close session |
| `terminal_input` | terminal | Send keystroke |
| `terminal_resize` | terminal | Resize PTY |
| `list_sftp_dir` | sftp | List remote directory |
| `sftp_download` | sftp | Download file (bytes) |
| `sftp_upload` | sftp | Upload file (bytes) |
| `sftp_mkdir` | sftp | Create directory |
| `sftp_rm` | sftp | Delete file |
| `sftp_rename` | sftp | Rename/move |
| `sftp_stat` | sftp | File metadata |
| `list_tunnels` | tunnels | List active tunnels |
| `create_tunnel` | tunnels | Create port forward |
| `stop_tunnel` | tunnels | Stop tunnel |
| `vault_status` | vault | Check lock state |
| `vault_unlock` | vault | Unlock with password |
| `vault_lock` | vault | Lock vault |
| `get_settings` | settings | Read app settings |
| `update_setting` | settings | Update a setting |

---

## 🤝 Contributing

1. Fork the repo
2. Create your branch (`git checkout -b feature/awesome-feature`)
3. Write your code and tests
4. Run `cargo tauri build` to verify
5. Commit with a clear message and open a PR

---

## 📄 License

This project is licensed under the **MIT License** — see the LICENSE file for details.