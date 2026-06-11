# React → Dioxus Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate Hermes Remote Manager from React + TypeScript to Dioxus (100% Rust) while keeping the Tauri backend.

**Architecture:** Modular Dioxus UI in `src-tauri/src/ui/` with signal-based state, termwiz terminal emulation, and direct function calls replacing Tauri IPC.

**Tech Stack:** Dioxus 0.6, termwiz 0.22, tokio channels, parking_lot mutexes

---

## Phase 1: Setup & Infrastructure (Days 1-2)

### Task 1.1: Add Dioxus Dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add dependencies to Cargo.toml**

```toml
[dependencies]
# ... existing deps ...
dioxus = { version = "0.6", features = ["desktop"] }
dioxus-router = "0.6"
termwiz = "0.22"
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check`
Expected: Compiles with new dependencies

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: add dioxus and termwiz dependencies"
```

---

### Task 1.2: Create UI Module Structure

**Files:**
- Create: `src-tauri/src/ui/mod.rs`
- Create: `src-tauri/src/ui/state.rs`
- Create: `src-tauri/src/ui/theme.rs`
- Create: `src-tauri/src/ui/pages/mod.rs`
- Create: `src-tauri/src/ui/pages/dashboard.rs`
- Create: `src-tauri/src/ui/pages/connections.rs`
- Create: `src-tauri/src/ui/pages/terminal_page.rs`
- Create: `src-tauri/src/ui/pages/settings.rs`
- Create: `src-tauri/src/ui/components/mod.rs`
- Create: `src-tauri/src/ui/components/layout/mod.rs`
- Create: `src-tauri/src/ui/components/layout/sidebar.rs`
- Create: `src-tauri/src/ui/components/layout/main_area.rs`
- Create: `src-tauri/src/ui/components/layout/status_bar.rs`
- Create: `src-tauri/src/ui/components/connections/mod.rs`
- Create: `src-tauri/src/ui/components/connections/connection_list.rs`
- Create: `src-tauri/src/ui/components/connections/connection_card.rs`
- Create: `src-tauri/src/ui/components/connections/connection_form.rs`
- Create: `src-tauri/src/ui/components/connections/folder_tree.rs`
- Create: `src-tauri/src/ui/components/terminal/mod.rs`
- Create: `src-tauri/src/ui/components/terminal/terminal_tab.rs`
- Create: `src-tauri/src/ui/components/terminal/terminal_session.rs`
- Create: `src-tauri/src/ui/components/terminal/sftp_browser.rs`
- Create: `src-tauri/src/ui/services/mod.rs`
- Create: `src-tauri/src/ui/services/ipc.rs`

- [ ] **Step 1: Create ui/mod.rs**

```rust
pub mod state;
pub mod theme;
pub mod pages;
pub mod components;
pub mod services;
```

- [ ] **Step 2: Create all placeholder files**

Each file starts with minimal content:

```rust
// Example: ui/pages/dashboard.rs
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        div { "Dashboard - TODO" }
    }
}
```

Repeat for all files with appropriate component names.

- [ ] **Step 3: Register ui module in lib.rs**

Add to `src-tauri/src/lib.rs`:
```rust
pub mod ui;
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/ui/
git commit -m "chore: create ui module structure"
```

---

### Task 1.3: Create AppState with Signals

**Files:**
- Create: `src-tauri/src/ui/state.rs`

- [ ] **Step 1: Write state.rs**

```rust
use dioxus::prelude::*;
use std::sync::Arc;
use parking_lot::Mutex;
use crate::storage::database::Database;
use crate::storage::models::{Connection, Folder};
use crate::ssh::session::{SessionManager, TerminalSession};
use crate::ssh::tunnels::TunnelManager;
use crate::security::vault::Vault;
use crate::settings::SettingsManager;

#[derive(Clone)]
pub struct AppState {
    // Persistent backend state
    pub db: Arc<Mutex<Database>>,
    pub vault: Arc<Mutex<Vault>>,
    pub session_manager: Arc<Mutex<SessionManager>>,
    pub tunnel_manager: Arc<Mutex<TunnelManager>>,
    pub settings: Arc<Mutex<SettingsManager>>,

    // Reactive UI state
    pub connections: Signal<Vec<Connection>>,
    pub folders: Signal<Vec<Folder>>,
    pub open_sessions: Signal<Vec<TerminalSession>>,
    pub active_session_id: Signal<Option<String>>,
    pub vault_unlocked: Signal<bool>,
    pub theme_mode: Signal<ThemeMode>,
    pub sidebar_collapsed: Signal<bool>,
    pub search_term: Signal<String>,
    pub filter_type: Signal<String>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl AppState {
    pub fn new(
        db: Arc<Mutex<Database>>,
        vault: Arc<Mutex<Vault>>,
        session_manager: Arc<Mutex<SessionManager>>,
        tunnel_manager: Arc<Mutex<TunnelManager>>,
        settings: Arc<Mutex<SettingsManager>>,
    ) -> Self {
        Self {
            db,
            vault,
            session_manager,
            tunnel_manager,
            settings,
            connections: use_signal(Vec::new),
            folders: use_signal(Vec::new),
            open_sessions: use_signal(Vec::new),
            active_session_id: use_signal(|| None),
            vault_unlocked: use_signal(|| false),
            theme_mode: use_signal(|| ThemeMode::Dark),
            sidebar_collapsed: use_signal(|| false),
            search_term: use_signal(String::new),
            filter_type: use_signal(|| "all".to_string()),
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles (may have warnings about unused fields)

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/ui/state.rs
git commit -m "feat: add AppState with Dioxus signals"
```

---

### Task 1.4: Create EventBridge for Terminal Events

**Files:**
- Create: `src-tauri/src/ui/services/event_bridge.rs`
- Modify: `src-tauri/src/ui/services/mod.rs`

- [ ] **Step 1: Write event_bridge.rs**

```rust
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct EventBridge {
    pub terminal_output: broadcast::Sender<(String, String)>,
    pub terminal_connected: broadcast::Sender<(String, u16, u16)>,
    pub terminal_error: broadcast::Sender<(String, String)>,
    pub terminal_exit: broadcast::Sender<String>,
}

impl EventBridge {
    pub fn new() -> Self {
        let (terminal_output, _) = broadcast::channel(256);
        let (terminal_connected, _) = broadcast::channel(32);
        let (terminal_error, _) = broadcast::channel(32);
        let (terminal_exit, _) = broadcast::channel(32);

        Self {
            terminal_output,
            terminal_connected,
            terminal_error,
            terminal_exit,
        }
    }
}
```

- [ ] **Step 2: Update services/mod.rs**

```rust
pub mod ipc;
pub mod event_bridge;

pub use event_bridge::EventBridge;
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/ui/services/
git commit -m "feat: add EventBridge for terminal events"
```

---

### Task 1.5: Create Routing

**Files:**
- Modify: `src-tauri/src/ui/mod.rs`
- Modify: `src-tauri/src/ui/pages/dashboard.rs`
- Modify: `src-tauri/src/ui/pages/connections.rs`
- Modify: `src-tauri/src/ui/pages/terminal_page.rs`
- Modify: `src-tauri/src/ui/pages/settings.rs`

- [ ] **Step 1: Update ui/mod.rs with routing**

```rust
use dioxus::prelude::*;
use dioxus_router::prelude::*;

pub mod state;
pub mod theme;
pub mod pages;
pub mod components;
pub mod services;

use pages::dashboard::Dashboard;
use pages::connections::Connections;
use pages::terminal_page::TerminalPage;
use pages::settings::Settings;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[route("/")]
    Dashboard {},
    #[route("/connections")]
    Connections {},
    #[route("/settings")]
    Settings {},
    #[route("/terminal/:session_id")]
    TerminalPage { session_id: String },
}

#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
```

- [ ] **Step 2: Update each page file**

Each page gets a proper component:

```rust
// dashboard.rs
use dioxus::prelude::*;
use super::super::Route;

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        div { class: "p-6",
            h1 { class: "text-2xl font-bold", "Dashboard" }
        }
    }
}
```

Similar for Connections, TerminalPage, Settings.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/ui/
git commit -m "feat: add Dioxus routing"
```

---

## Phase 2: Layout Components (Days 3-4)

### Task 2.1: Create Sidebar Component

**Files:**
- Modify: `src-tauri/src/ui/components/layout/sidebar.rs`

- [ ] **Step 1: Write sidebar.rs**

```rust
use dioxus::prelude::*;
use crate::ui::{Route, state::AppState};

#[component]
pub fn Sidebar() -> Element {
    let state = use_context::<AppState>();
    let collapsed = state.sidebar_collapsed.read();

    rsx! {
        aside { class: if *collapsed { "w-16 bg-gray-900" } else { "w-64 bg-gray-900" },
            div { class: "p-4",
                if !*collapsed {
                    h1 { class: "text-xl font-bold text-white", "Hermes" }
                }
            }
            nav { class: "mt-4",
                Link::<Route> { to: Route::Dashboard {},
                    div { class: "px-4 py-2 text-gray-300 hover:bg-gray-800", "Dashboard" }
                }
                Link::<Route> { to: Route::Connections {},
                    div { class: "px-4 py-2 text-gray-300 hover:bg-gray-800", "Connections" }
                }
                Link::<Route> { to: Route::Settings {},
                    div { class: "px-4 py-2 text-gray-300 hover:bg-gray-800", "Settings" }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/components/layout/sidebar.rs
git commit -m "feat: add Sidebar component"
```

---

### Task 2.2: Create StatusBar Component

**Files:**
- Modify: `src-tauri/src/ui/components/layout/status_bar.rs`

- [ ] **Step 1: Write status_bar.rs**

```rust
use dioxus::prelude::*;
use crate::ui::state::{AppState, ThemeMode};

#[component]
pub fn StatusBar() -> Element {
    let state = use_context::<AppState>();
    let sessions = state.open_sessions.read();
    let theme = state.theme_mode.read();
    let vault_unlocked = state.vault_unlocked.read();

    rsx! {
        footer { class: "h-8 bg-gray-900 border-t border-gray-800 flex items-center px-4 text-xs text-gray-400",
            span { "Sessions: {sessions.len()}" }
            span { class: "ml-4", "Vault: {if *vault_unlocked { "Unlocked" } else { "Locked" }}" }
            div { class: "ml-auto",
                button {
                    class: "text-gray-400 hover:text-white",
                    onclick: move |_| {
                        let mut theme = state.theme_mode.write();
                        *theme = match *theme {
                            ThemeMode::Dark => ThemeMode::Light,
                            ThemeMode::Light => ThemeMode::Dark,
                        };
                    },
                    if *theme == ThemeMode::Dark { "☀️" } else { "🌙" }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/components/layout/status_bar.rs
git commit -m "feat: add StatusBar component"
```

---

### Task 2.3: Create TerminalTab Component

**Files:**
- Modify: `src-tauri/src/ui/components/terminal/terminal_tab.rs`

- [ ] **Step 1: Write terminal_tab.rs**

```rust
use dioxus::prelude::*;
use crate::ui::state::AppState;

#[component]
pub fn TerminalTabs() -> Element {
    let state = use_context::<AppState>();
    let sessions = state.open_sessions.read();
    let active_id = state.active_session_id.read();

    if sessions.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "flex bg-gray-800 border-b border-gray-700",
            for session in sessions.iter() {
                div {
                    key: "{session.id}",
                    class: if Some(&session.id) == active_id.as_ref() {
                        "px-4 py-2 bg-gray-900 text-white border-b-2 border-blue-500"
                    } else {
                        "px-4 py-2 text-gray-400 hover:bg-gray-700"
                    },
                    onclick: move |_| {
                        *state.active_session_id.write() = Some(session.id.clone());
                    },
                    span { class: "mr-2", if session.connected { "🟢" } else { "🔴" } },
                    span { "{session.name}" }
                    button {
                        class: "ml-2 text-gray-500 hover:text-white",
                        onclick: move |e| {
                            e.stop_propagation();
                            // TODO: close session
                        },
                        "×"
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/components/terminal/terminal_tab.rs
git commit -m "feat: add TerminalTabs component"
```

---

### Task 2.4: Create Main Layout

**Files:**
- Modify: `src-tauri/src/ui/mod.rs`

- [ ] **Step 1: Update ui/mod.rs with layout**

```rust
// ... existing imports ...

use components::layout::sidebar::Sidebar;
use components::layout::status_bar::StatusBar;
use components::terminal::terminal_tab::TerminalTabs;

#[component]
pub fn App() -> Element {
    rsx! {
        div { class: "flex h-screen bg-gray-950 text-white",
            Sidebar {}
            div { class: "flex-1 flex flex-col",
                TerminalTabs {}
                main { class: "flex-1 overflow-auto",
                    Router::<Route> {}
                }
                StatusBar {}
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/mod.rs
git commit -m "feat: add main layout with sidebar and status bar"
```

---

## Phase 3: Dashboard Page (Day 5)

### Task 3.1: Create Dashboard Page

**Files:**
- Modify: `src-tauri/src/ui/pages/dashboard.rs`

- [ ] **Step 1: Write dashboard.rs**

```rust
use dioxus::prelude::*;
use crate::ui::state::AppState;

#[component]
pub fn Dashboard() -> Element {
    let state = use_context::<AppState>();
    let connections = state.connections.read();
    let sessions = state.open_sessions.read();

    let total = connections.len();
    let active = sessions.len();
    let favorites = connections.iter().filter(|c| c.is_favorite).count();

    rsx! {
        div { class: "p-6",
            h1 { class: "text-2xl font-bold mb-6", "Dashboard" }

            // Stats cards
            div { class: "grid grid-cols-3 gap-4 mb-6",
                div { class: "bg-gray-800 rounded-lg p-4",
                    div { class: "text-gray-400 text-sm", "Total Connections" }
                    div { class: "text-3xl font-bold", "{total}" }
                }
                div { class: "bg-gray-800 rounded-lg p-4",
                    div { class: "text-gray-400 text-sm", "Active Sessions" }
                    div { class: "text-3xl font-bold text-green-500", "{active}" }
                }
                div { class: "bg-gray-800 rounded-lg p-4",
                    div { class: "text-gray-400 text-sm", "Favorites" }
                    div { class: "text-3xl font-bold text-yellow-500", "{favorites}" }
                }
            }

            // Recent connections
            h2 { class: "text-lg font-semibold mb-4", "Recent Connections" }
            div { class: "grid grid-cols-2 gap-4",
                for conn in connections.iter().take(6) {
                    div { class: "bg-gray-800 rounded-lg p-4 hover:bg-gray-700 cursor-pointer",
                        div { class: "font-medium", "{conn.name}" }
                        div { class: "text-sm text-gray-400", "{conn.host}:{conn.port}" }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/pages/dashboard.rs
git commit -m "feat: add Dashboard page"
```

---

## Phase 4: Connections Page (Days 6-8)

### Task 4.1: Create ConnectionCard Component

**Files:**
- Modify: `src-tauri/src/ui/components/connections/connection_card.rs`

- [ ] **Step 1: Write connection_card.rs**

```rust
use dioxus::prelude::*;
use crate::storage::models::Connection;

#[component]
pub fn ConnectionCard(conn: Connection, on_connect: EventHandler<String>) -> Element {
    let type_icon = match conn.r#type.as_str() {
        "ssh" => "🖥️",
        "rdp" => "🖥️",
        "serial" => "🔌",
        _ => "📡",
    };

    rsx! {
        div { class: "bg-gray-800 rounded-lg p-4 hover:bg-gray-700 cursor-pointer group",
            onclick: move |_| on_connect.call(conn.id.clone()),

            div { class: "flex items-center justify-between mb-2",
                span { class: "text-lg", "{type_icon}" }
                if conn.is_favorite {
                    span { class: "text-yellow-500", "⭐" }
                }
            }
            div { class: "font-medium", "{conn.name}" }
            div { class: "text-sm text-gray-400", "{conn.host}:{conn.port}" }
            if let Some(tags) = &conn.tags {
                div { class: "mt-2 flex gap-1 flex-wrap",
                    for tag in tags.split(',').take(3) {
                        span { class: "px-2 py-0.5 bg-gray-700 rounded text-xs", "{tag}" }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/components/connections/connection_card.rs
git commit -m "feat: add ConnectionCard component"
```

---

### Task 4.2: Create FolderTree Component

**Files:**
- Modify: `src-tauri/src/ui/components/connections/folder_tree.rs`

- [ ] **Step 1: Write folder_tree.rs**

```rust
use dioxus::prelude::*;
use crate::storage::models::Folder;

#[component]
pub fn FolderTree(
    folders: Vec<Folder>,
    selected_id: Option<String>,
    on_select: EventHandler<String>,
) -> Element {
    let root_folders: Vec<_> = folders.iter()
        .filter(|f| f.parent_id.is_none())
        .collect();

    rsx! {
        div { class: "space-y-1",
            for folder in root_folders {
                FolderNode {
                    key: "{folder.id}",
                    folder: folder.clone(),
                    all_folders: folders.clone(),
                    selected_id: selected_id.clone(),
                    on_select: on_select,
                    depth: 0,
                }
            }
        }
    }
}

#[component]
fn FolderNode(
    folder: Folder,
    all_folders: Vec<Folder>,
    selected_id: Option<String>,
    on_select: EventHandler<String>,
    depth: u32,
) -> Element {
    let mut expanded = use_signal(|| false);
    let children: Vec<_> = all_folders.iter()
        .filter(|f| f.parent_id.as_deref() == Some(&folder.id))
        .collect();

    let indent = depth * 16;
    let has_children = !children.is_empty();
    let is_selected = Some(&folder.id) == selected_id.as_ref();

    rsx! {
        div {
            div {
                class: if is_selected {
                    "flex items-center px-2 py-1 bg-blue-600 rounded cursor-pointer"
                } else {
                    "flex items-center px-2 py-1 hover:bg-gray-800 rounded cursor-pointer"
                },
                style: "padding-left: {indent}px",
                onclick: move |_| on_select.call(folder.id.clone()),

                if has_children {
                    span {
                        class: "mr-1 text-gray-400",
                        onclick: move |e| {
                            e.stop_propagation();
                            *expanded.write() = !*expanded;
                        },
                        if *expanded { "▼" } else { "▶" }
                    }
                }
                span { class: "mr-2", "📁" }
                span { "{folder.name}" }
            }
            if *expanded && has_children {
                for child in children {
                    FolderNode {
                        key: "{child.id}",
                        folder: child.clone(),
                        all_folders: all_folders.clone(),
                        selected_id: selected_id.clone(),
                        on_select: on_select,
                        depth: depth + 1,
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/components/connections/folder_tree.rs
git commit -m "feat: add FolderTree component with recursive rendering"
```

---

### Task 4.3: Create ConnectionForm Component

**Files:**
- Modify: `src-tauri/src/ui/components/connections/connection_form.rs`

- [ ] **Step 1: Write connection_form.rs (simplified)**

```rust
use dioxus::prelude::*;
use crate::storage::models::{Connection, Folder};

#[derive(Clone, PartialEq)]
pub struct ConnectionFormData {
    pub name: String,
    pub conn_type: String,
    pub folder_id: Option<String>,
    pub host: String,
    pub port: String,
    pub username: String,
    pub auth_type: String,
    pub tags: String,
    pub notes: String,
    pub is_favorite: bool,
}

impl Default for ConnectionFormData {
    fn default() -> Self {
        Self {
            name: String::new(),
            conn_type: "ssh".to_string(),
            folder_id: None,
            host: String::new(),
            port: "22".to_string(),
            username: String::new(),
            auth_type: "password".to_string(),
            tags: String::new(),
            notes: String::new(),
            is_favorite: false,
        }
    }
}

#[component]
pub fn ConnectionForm(
    editing: Option<Connection>,
    folders: Vec<Folder>,
    on_submit: EventHandler<ConnectionFormData>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut form = use_signal(|| {
        editing.map(|c| ConnectionFormData {
            name: c.name,
            conn_type: c.r#type,
            folder_id: c.folder_id,
            host: c.host,
            port: c.port.to_string(),
            username: c.username,
            auth_type: c.auth_type,
            tags: c.tags.unwrap_or_default(),
            notes: c.notes.unwrap_or_default(),
            is_favorite: c.is_favorite,
        }).unwrap_or_default()
    });

    rsx! {
        div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            div { class: "bg-gray-800 rounded-lg p-6 w-full max-w-md",
                h2 { class: "text-xl font-bold mb-4",
                    if editing.is_some() { "Edit Connection" } else { "New Connection" }
                }

                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm text-gray-400 mb-1", "Name" }
                        input {
                            class: "w-full bg-gray-700 rounded px-3 py-2",
                            value: "{form.read().name}",
                            oninput: move |e| form.write().name = e.value(),
                        }
                    }
                    div {
                        label { class: "block text-sm text-gray-400 mb-1", "Host" }
                        input {
                            class: "w-full bg-gray-700 rounded px-3 py-2",
                            value: "{form.read().host}",
                            oninput: move |e| form.write().host = e.value(),
                        }
                    }
                    div {
                        label { class: "block text-sm text-gray-400 mb-1", "Port" }
                        input {
                            class: "w-full bg-gray-700 rounded px-3 py-2",
                            value: "{form.read().port}",
                            oninput: move |e| form.write().port = e.value(),
                        }
                    }
                    div {
                        label { class: "block text-sm text-gray-400 mb-1", "Username" }
                        input {
                            class: "w-full bg-gray-700 rounded px-3 py-2",
                            value: "{form.read().username}",
                            oninput: move |e| form.write().username = e.value(),
                        }
                    }
                }

                div { class: "flex justify-end gap-2 mt-6",
                    button {
                        class: "px-4 py-2 bg-gray-700 rounded hover:bg-gray-600",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 rounded hover:bg-blue-500",
                        onclick: move |_| on_submit.call(form.read().clone()),
                        if editing.is_some() { "Update" } else { "Create" }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/components/connections/connection_form.rs
git commit -m "feat: add ConnectionForm component"
```

---

### Task 4.4: Create Connections Page

**Files:**
- Modify: `src-tauri/src/ui/pages/connections.rs`

- [ ] **Step 1: Write connections.rs**

```rust
use dioxus::prelude::*;
use crate::ui::state::AppState;
use crate::ui::components::connections::folder_tree::FolderTree;
use crate::ui::components::connections::connection_card::ConnectionCard;
use crate::ui::components::connections::connection_form::{ConnectionForm, ConnectionFormData};

#[component]
pub fn Connections() -> Element {
    let state = use_context::<AppState>();
    let connections = state.connections.read();
    let folders = state.folders.read();
    let search = state.search_term.read();
    let filter = state.filter_type.read();

    let mut show_form = use_signal(|| false);
    let mut editing_conn = use_signal(|| None::<crate::storage::models::Connection>);
    let mut selected_folder = use_signal(|| None::<String>);

    let filtered: Vec<_> = connections.iter()
        .filter(|c| {
            let matches_search = search.is_empty() ||
                c.name.to_lowercase().contains(&search.to_lowercase()) ||
                c.host.to_lowercase().contains(&search.to_lowercase());
            let matches_filter = *filter == "all" || c.r#type == *filter;
            let matches_folder = selected_folder.read().as_ref() == c.folder_id.as_ref();
            matches_search && matches_filter && matches_folder
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "flex h-full",
            // Left panel - Folder tree
            div { class: "w-64 bg-gray-900 border-r border-gray-800 p-4",
                h3 { class: "text-sm font-semibold text-gray-400 mb-2", "Folders" }
                FolderTree {
                    folders: folders.clone(),
                    selected_id: (*selected_folder.read()).clone(),
                    on_select: move |id: String| {
                        *selected_folder.write() = Some(id);
                    },
                }
            }

            // Right panel - Connection list
            div { class: "flex-1 p-4",
                div { class: "flex items-center justify-between mb-4",
                    input {
                        class: "bg-gray-800 rounded px-3 py-2 w-64",
                        placeholder: "Search connections...",
                        value: "{search}",
                        oninput: move |e| *state.search_term.write() = e.value(),
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 rounded hover:bg-blue-500",
                        onclick: move |_| {
                            editing_conn.write().take();
                            *show_form.write() = true;
                        },
                        "+ New Connection"
                    }
                }

                div { class: "grid grid-cols-3 gap-4",
                    for conn in filtered.iter() {
                        ConnectionCard {
                            key: "{conn.id}",
                            conn: conn.clone(),
                            on_connect: move |id: String| {
                                // TODO: initiate SSH connection
                            },
                        }
                    }
                }
            }

            // Connection form modal
            if *show_form {
                ConnectionForm {
                    editing: (*editing_conn.read()).clone(),
                    folders: folders.clone(),
                    on_submit: move |data: ConnectionFormData| {
                        // TODO: save connection
                        *show_form.write() = false;
                    },
                    on_cancel: move |_| {
                        *show_form.write() = false;
                    },
                }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/pages/connections.rs
git commit -m "feat: add Connections page"
```

---

## Phase 5: Terminal Page (Days 9-12)

### Task 5.1: Create Terminal Emulator Wrapper

**Files:**
- Create: `src-tauri/src/ui/components/terminal/terminal_emulator.rs`
- Modify: `src-tauri/src/ui/components/terminal/mod.rs`

- [ ] **Step 1: Write terminal_emulator.rs**

```rust
use termwiz::terminal::{Terminal, ScreenSize};
use termwiz::cell::{CellAttributes, Cell};
use termwiz::surface::{Surface, SurfaceBuffer, Change};
use termwiz::escape::{Action, CSI};
use std::io::Write;

pub struct TerminalEmulator {
    surface: Surface,
    screen_size: ScreenSize,
}

impl TerminalEmulator {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            surface: Surface::new(cols as usize, rows as usize),
            screen_size: ScreenSize {
                rows,
                cols,
                xpixel: 0,
                ypixel: 0,
            },
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.surface.resize(cols as usize, rows as usize);
        self.screen_size.rows = rows;
        self.screen_size.cols = cols;
    }

    pub fn process_output(&mut self, data: &str) {
        let mut parser = termwiz::escape::Parser::new();
        let actions = parser.parse(data);

        for action in actions {
            match action {
                Action::Print(ch) => {
                    self.surface.add_change(Change::Print(ch));
                }
                Action::CSI(csi) => {
                    match csi {
                        CSI::Cursor { params, .. } => {
                            // Handle cursor movement
                            if let Some((row, col)) = parse_cursor_position(&params) {
                                self.surface.add_change(Change::CursorPosition {
                                    x: col,
                                    y: row,
                                    mode: termwiz::surface::CursorPositionMode::Absolute,
                                });
                            }
                        }
                        CSI::Edit(params) => {
                            // Handle edit operations (clear screen, etc.)
                            if let Some(code) = params.first() {
                                match code {
                                    2 => self.surface.add_change(Change::ClearScreen),
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        self.surface.add_change(Change::Flush);
    }

    pub fn render(&self) -> String {
        let buffer = self.surface.buffer();
        let mut output = String::new();

        for row in 0..buffer.get_height() {
            for col in 0..buffer.get_width() {
                if let Some(cell) = buffer.get_cell(col, row) {
                    output.push(cell.chars().next().unwrap_or(' '));
                }
            }
            output.push('\n');
        }
        output
    }

    pub fn get_size(&self) -> (u16, u16) {
        (self.screen_size.cols, self.screen_size.rows)
    }
}

fn parse_cursor_position(params: &[i32]) -> Option<(usize, usize)> {
    let row = params.first().copied().unwrap_or(1).max(1) as usize - 1;
    let col = params.get(1).copied().unwrap_or(1).max(1) as usize - 1;
    Some((row, col))
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/components/terminal/terminal_emulator.rs
git commit -m "feat: add TerminalEmulator with termwiz"
```

---

### Task 5.2: Create TerminalSession Component

**Files:**
- Modify: `src-tauri/src/ui/components/terminal/terminal_session.rs`

- [ ] **Step 1: Write terminal_session.rs**

```rust
use dioxus::prelude::*;
use crate::ui::state::AppState;
use super::terminal_emulator::TerminalEmulator;

#[component]
pub fn TerminalSession(session_id: String) -> Element {
    let state = use_context::<AppState>();
    let mut emulator = use_signal(|| TerminalEmulator::new(80, 24));
    let mut output_text = use_signal(String::new);

    // TODO: Connect to SSH session via EventBridge
    // TODO: Handle terminal input
    // TODO: Handle resize

    rsx! {
        div { class: "h-full bg-black p-2 font-mono text-sm text-green-500 overflow-auto",
            pre { class: "whitespace-pre-wrap", "{output_text}" }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/components/terminal/terminal_session.rs
git commit -m "feat: add TerminalSession component shell"
```

---

### Task 5.3: Create TerminalPage

**Files:**
- Modify: `src-tauri/src/ui/pages/terminal_page.rs`

- [ ] **Step 1: Write terminal_page.rs**

```rust
use dioxus::prelude::*;
use crate::ui::state::AppState;
use crate::ui::components::terminal::terminal_session::TerminalSession;
use crate::ui::components::terminal::sftp_browser::SftpBrowser;

#[component]
pub fn TerminalPage(session_id: String) -> Element {
    let state = use_context::<AppState>();
    let sessions = state.open_sessions.read();
    let session = sessions.iter().find(|s| s.id == session_id);

    let mut show_sftp = use_signal(|| false);

    match session {
        Some(session) => rsx! {
            div { class: "flex h-full",
                div { class: if *show_sftp { "flex-1" } else { "w-full" },
                    TerminalSession { session_id: session_id.clone() }
                }
                if *show_sftp {
                    SftpBrowser { session_id: session_id.clone() }
                }
            }
        },
        None => rsx! {
            div { class: "flex items-center justify-center h-full text-gray-400",
                "Session not found"
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/pages/terminal_page.rs
git commit -m "feat: add TerminalPage"
```

---

## Phase 6: SFTP Browser (Days 13-15)

### Task 6.1: Create SftpBrowser Component

**Files:**
- Modify: `src-tauri/src/ui/components/terminal/sftp_browser.rs`

- [ ] **Step 1: Write sftp_browser.rs (simplified)**

```rust
use dioxus::prelude::*;
use crate::ui::state::AppState;

#[derive(Clone)]
struct SftpEntry {
    name: String,
    is_dir: bool,
    size: i64,
    permissions: String,
}

#[component]
pub fn SftpBrowser(session_id: String) -> Element {
    let state = use_context::<AppState>();
    let mut current_path = use_signal(|| "/home".to_string());
    let mut entries = use_signal(|| Vec::<SftpEntry>::new());
    let mut loading = use_signal(|| false);

    // TODO: Load directory contents via IPC

    rsx! {
        div { class: "w-80 bg-gray-900 border-l border-gray-800 flex flex-col",
            // Breadcrumb
            div { class: "p-2 border-b border-gray-800 text-sm",
                span { class: "text-gray-400", "Remote: " }
                span { "{current_path}" }
            }

            // File list
            div { class: "flex-1 overflow-auto",
                if *loading {
                    div { class: "p-4 text-center text-gray-400", "Loading..." }
                } else {
                    for entry in entries.iter() {
                        div {
                            key: "{entry.name}",
                            class: "flex items-center px-3 py-2 hover:bg-gray-800 cursor-pointer",
                            onclick: move |_| {
                                if entry.is_dir {
                                    // Navigate into directory
                                }
                            },
                            span { class: "mr-2", if entry.is_dir { "📁" } else { "📄" } }
                            span { class: "flex-1", "{entry.name}" }
                            span { class: "text-xs text-gray-500", "{format_size(entry.size)}" }
                        }
                    }
                }
            }
        }
    }
}

fn format_size(bytes: i64) -> String {
    if bytes < 1024 { return format!("{} B", bytes); }
    if bytes < 1024 * 1024 { return format!("{:.1} KB", bytes as f64 / 1024.0); }
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/components/terminal/sftp_browser.rs
git commit -m "feat: add SftpBrowser component"
```

---

## Phase 7: Settings Page (Day 16)

### Task 7.1: Create Settings Page

**Files:**
- Modify: `src-tauri/src/ui/pages/settings.rs`

- [ ] **Step 1: Write settings.rs**

```rust
use dioxus::prelude::*;
use crate::ui::state::{AppState, ThemeMode};

#[component]
pub fn Settings() -> Element {
    let state = use_context::<AppState>();
    let vault_unlocked = state.vault_unlocked.read();
    let theme = state.theme_mode.read();

    let mut active_tab = use_signal(|| "general".to_string());
    let mut password = use_signal(String::new);

    rsx! {
        div { class: "p-6 max-w-4xl mx-auto",
            h1 { class: "text-2xl font-bold mb-6", "Settings" }

            // Tabs
            div { class: "flex gap-4 border-b border-gray-800 mb-6",
                for tab in ["general", "appearance", "security", "terminal", "about"] {
                    button {
                        key: "{tab}",
                        class: if *active_tab == tab {
                            "pb-2 border-b-2 border-blue-500 text-white"
                        } else {
                            "pb-2 text-gray-400 hover:text-white"
                        },
                        onclick: move |_| *active_tab.write() = tab.to_string(),
                        "{tab}"
                    }
                }
            }

            // Tab content
            match active_tab.read().as_str() {
                "general" => rsx! {
                    div { "General settings coming soon..." }
                },
                "appearance" => rsx! {
                    div { class: "space-y-4",
                        h3 { class: "font-semibold", "Theme" }
                        div { class: "flex gap-2",
                            button {
                                class: if *theme == ThemeMode::Dark {
                                    "px-4 py-2 bg-blue-600 rounded"
                                } else {
                                    "px-4 py-2 bg-gray-700 rounded"
                                },
                                onclick: move |_| *state.theme_mode.write() = ThemeMode::Dark,
                                "Dark"
                            }
                            button {
                                class: if *theme == ThemeMode::Light {
                                    "px-4 py-2 bg-blue-600 rounded"
                                } else {
                                    "px-4 py-2 bg-gray-700 rounded"
                                },
                                onclick: move |_| *state.theme_mode.write() = ThemeMode::Light,
                                "Light"
                            }
                        }
                    }
                },
                "security" => rsx! {
                    div { class: "space-y-4",
                        h3 { class: "font-semibold", "Vault" }
                        if *vault_unlocked {
                            p { class: "text-green-500", "Vault is unlocked" }
                            button {
                                class: "px-4 py-2 bg-red-600 rounded",
                                onclick: move |_| {
                                    // TODO: lock vault
                                },
                                "Lock Vault"
                            }
                        } else {
                            p { class: "text-yellow-500", "Vault is locked" }
                            input {
                                class: "bg-gray-700 rounded px-3 py-2",
                                r#type: "password",
                                placeholder: "Master password",
                                value: "{password}",
                                oninput: move |e| *password.write() = e.value(),
                            }
                            button {
                                class: "px-4 py-2 bg-blue-600 rounded mt-2",
                                onclick: move |_| {
                                    // TODO: unlock vault
                                },
                                "Unlock"
                            }
                        }
                    }
                },
                "terminal" => rsx! {
                    div { "Terminal settings coming soon..." }
                },
                "about" => rsx! {
                    div {
                        h3 { class: "font-semibold", "Hermes Remote Manager" }
                        p { class: "text-gray-400", "Version 0.1.0" }
                    }
                },
                _ => rsx! { div { "Unknown tab" } }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/ui/pages/settings.rs
git commit -m "feat: add Settings page"
```

---

## Phase 8: Integration & Polish (Days 17-20)

### Task 8.1: Wire Up Dioxus Launch in lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Update lib.rs to launch Dioxus**

```rust
// ... existing code ...

use crate::ui::{App, state::AppState};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // ... existing setup code ...

            // Create Dioxus app
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                dioxus::launch(App);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // ... existing handlers ...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: wire up Dioxus launch in lib.rs"
```

---

### Task 8.2: Remove React Frontend

**Files:**
- Delete: `src/` (entire directory)
- Delete: `package.json`
- Delete: `package-lock.json`
- Delete: `vite.config.ts`
- Delete: `tsconfig.json`
- Delete: `tsconfig.node.json`
- Delete: `postcss.config.js`
- Delete: `tailwind.config.js`
- Delete: `index.html`

- [ ] **Step 1: Remove files**

```bash
rm -rf src/
rm -f package.json package-lock.json vite.config.ts tsconfig.json tsconfig.node.json postcss.config.js tailwind.config.js index.html
```

- [ ] **Step 2: Commit**

```bash
git add -A
git commit -m "chore: remove React frontend files"
```

---

### Task 8.3: Update Tauri Config

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Update tauri.conf.json**

Remove webview-related config, keep only Rust-side settings.

- [ ] **Step 2: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: update Tauri config for Dioxus"
```

---

### Task 8.4: Final Build & Test

- [ ] **Step 1: Build the app**

Run: `cargo build --release`
Expected: Builds successfully

- [ ] **Step 2: Run the app**

Run: `cargo run`
Expected: App launches with Dioxus UI

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "chore: final integration and testing"
```

---

## Summary

| Phase | Tasks | Estimated Time |
|-------|-------|----------------|
| Phase 1: Setup & Infrastructure | 5 tasks | 2 days |
| Phase 2: Layout Components | 4 tasks | 2 days |
| Phase 3: Dashboard Page | 1 task | 1 day |
| Phase 4: Connections Page | 4 tasks | 3 days |
| Phase 5: Terminal Page | 3 tasks | 4 days |
| Phase 6: SFTP Browser | 1 task | 3 days |
| Phase 7: Settings Page | 1 task | 1 day |
| Phase 8: Integration & Polish | 4 tasks | 4 days |
| **Total** | **23 tasks** | **~20 days** |
