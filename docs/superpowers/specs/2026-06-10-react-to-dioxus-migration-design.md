# React → Dioxus Migration Design

## Overview

Migrate Hermes Remote Manager from React + TypeScript frontend to Dioxus (Rust UI framework) while keeping the existing Tauri backend. The goal is 100% Rust codebase.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| UI Framework | Dioxus | Cross-platform, Tauri integration, signal-based state |
| Terminal Emulation | Native Rust (termwiz) | 100% Rust, no webview dependency |
| Migration Strategy | Full rewrite | Clean break, no legacy code |
| Architecture | Modular (src/ui/) | Clean separation, maintainable |
| State Management | Dioxus Signals | Native, simple, reactive |

## Architecture

### Directory Structure

```
src-tauri/src/
├── main.rs                    # Entry point (modified)
├── lib.rs                     # App bootstrap + Dioxus launch (modified)
├── error.rs                   # AppError (keep)
├── logging.rs                 # Logging (keep)
├── settings.rs                # SettingsManager (keep)
├── commands/                  # Refactored for direct calls (keep)
├── storage/                   # Database layer (keep)
├── ssh/                       # SSH/SFTP/Tunnel (keep)
├── security/                  # Crypto/Vault (keep)
└── ui/                        # NEW: Dioxus UI layer
    ├── mod.rs                 # App component + routing
    ├── state.rs               # Global state (signals)
    ├── theme.rs               # Theme provider
    ├── pages/
    │   ├── mod.rs
    │   ├── dashboard.rs
    │   ├── connections.rs
    │   ├── terminal_page.rs
    │   └── settings.rs
    ├── components/
    │   ├── mod.rs
    │   ├── layout/
    │   │   ├── sidebar.rs
    │   │   ├── main_area.rs
    │   │   └── status_bar.rs
    │   ├── connections/
    │   │   ├── connection_list.rs
    │   │   ├── connection_card.rs
    │   │   ├── connection_form.rs
    │   │   └── folder_tree.rs
    │   └── terminal/
    │       ├── terminal_tab.rs
    │       ├── terminal_session.rs
    │       └── sftp_browser.rs
    └── services/
        ├── mod.rs
        └── ipc.rs             # Direct function calls
```

### State Management

```rust
// ui/state.rs
use dioxus::prelude::*;
use std::sync::Arc;
use parking_lot::Mutex;
use crate::storage::database::Database;
use crate::ssh::session::SessionManager;
use crate::ssh::tunnels::TunnelManager;
use crate::security::vault::Vault;
use crate::settings::SettingsManager;

#[derive(Clone)]
pub struct AppState {
    // Persistent state (backend)
    pub db: Arc<Mutex<Database>>,
    pub vault: Arc<Mutex<Vault>>,
    pub sessions: Arc<Mutex<SessionManager>>,
    pub tunnels: Arc<Mutex<TunnelManager>>,
    pub settings: Arc<Mutex<SettingsManager>>,

    // Reactive UI state (signals)
    pub connections: Signal<Vec<Connection>>,
    pub folders: Signal<Vec<Folder>>,
    pub open_sessions: Signal<Vec<TerminalSession>>,
    pub active_session_id: Signal<Option<String>>,
    pub vault_unlocked: Signal<bool>,
    pub theme: Signal<ThemeMode>,
    pub sidebar_collapsed: Signal<bool>,
}
```

### Routing

```rust
// ui/mod.rs
#[derive(Routable, Clone)]
enum Route {
    #[route("/")]
    Dashboard,
    #[route("/connections")]
    Connections,
    #[route("/settings")]
    Settings,
    #[route("/terminal/:session_id")]
    TerminalPage { session_id: String },
}
```

### Terminal Emulation

- Library: `termwiz` (from WezTerm) for VT sequence parsing
- Rendering: Use `termwiz::surface::Surface` to maintain terminal state, then render as styled `<span>` elements in Dioxus
- Each cell maps to a `<span>` with foreground/background colors via inline CSS
- Cursor rendered as a separate element with absolute positioning
- Scrollback buffer maintained in Surface (configurable lines, default 10000)
- I/O flow: SSH thread → output buffer → termwiz parser → surface → Dioxus re-render

### Services Layer

Replace Tauri IPC with direct function calls:

```rust
// ui/services/ipc.rs
pub async fn list_connections(state: &AppState) -> AppResult<Vec<Connection>> {
    let db = state.db.lock();
    commands::connections::list_connections_internal(&db)
}
```

Event system: Replace Tauri events with `tokio::sync::broadcast` channels.

### Theme

Keep CSS variables approach:
- Dark/Light mode stored in signal
- CSS classes toggled on root element
- Components use `class="bg-primary text-primary"`

## Implementation Phases

### Phase 1: Setup & Infrastructure (Days 1-2)
- Add Dioxus dependencies to Cargo.toml
- Create ui/ module structure
- Setup routing
- Create AppState with signals
- Create EventBridge for terminal events

### Phase 2: Layout Components (Days 3-4)
- Sidebar component
- MainArea component
- StatusBar component
- TerminalTab component
- Theme toggle

### Phase 3: Dashboard Page (Day 5)
- Stats cards
- Quick connect bar
- Recent connections grid

### Phase 4: Connections Page (Days 6-8)
- FolderTree component (recursive)
- ConnectionList component
- ConnectionCard component
- ConnectionForm modal (complex form)

### Phase 5: Terminal Page (Days 9-12)
- TerminalWidget (termwiz integration)
- TerminalSession component
- SSH I/O integration
- Output buffering
- Resize handling

### Phase 6: SFTP Browser (Days 13-15)
- Directory listing
- Breadcrumb navigation
- File operations (upload/download/delete/rename)
- Context menu

### Phase 7: Settings Page (Day 16)
- Tabbed interface (5 tabs)
- Vault lock/unlock
- Theme settings

### Phase 8: Integration & Polish (Days 17-20)
- Remove React/Vite/Node.js dependencies
- Remove src/ directory
- Remove package.json
- Update Tauri config
- End-to-end testing
- Performance optimization

## Dependencies to Add

```toml
[dependencies]
dioxus = { version = "0.6", features = ["desktop"] }
dioxus-router = "0.6"
termwiz = "0.22"
tokio = { version = "1", features = ["sync"] }
```

## Files to Delete

- `src/` (entire React frontend)
- `package.json`
- `package-lock.json`
- `node_modules/`
- `vite.config.ts`
- `tsconfig.json`
- `tsconfig.node.json`
- `postcss.config.js`
- `tailwind.config.js`
- `index.html`

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Terminal emulation complexity | Use termwiz (battle-tested from WezTerm) |
| Dioxus learning curve | Start with simple components, iterate |
| Performance regression | Benchmark terminal rendering early |
| Missing xterm.js features | termwiz supports most VT100/xterm sequences |

## Success Criteria

1. All existing functionality preserved
2. No JavaScript/TypeScript in codebase
3. Terminal emulation works for SSH sessions
4. App builds and runs on Windows
5. Performance comparable to React version
