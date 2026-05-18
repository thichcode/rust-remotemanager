# Architecture — Hermes Remote Manager

## Terminal Output Flow

### The Race Condition Problem

When a user connects via SSH, the Rust I/O thread starts emitting `terminal:output` events as soon as the SSH connection succeeds. The old architecture had the output listener registered inside `useTerminal.ts` (inside child component `useEffect`), which runs AFTER the parent component mounts. Any output emitted during that gap was permanently lost.

### Solution: Rust-Side Output Buffering

```
Timeline (new flow):
────────────────────────────────────────────────────────────────────────────
T0  connectSSH() returns sessionId
T1  initBuffer(sessionId)                          [JS: create buffer]
T2  listenToTerminalOutput(sessionId, callback)   [JS: register Tauri listener]
    └─ Tauri IPC roundtrip starts (async)
T3  [navigate to /terminal/sessionId]
    └─ React renders TerminalPage → useTerminal → xterm opens
T4  Tauri listener registration completes (.then())
T5  flushSessionOutput(sessionId)
    └─ Rust sets output_ready=true
    └─ Rust drains VecDeque → returns Vec<String> to JS
T6  JS writes flushed data to buffer
T7  setWriter(sessionId, writer)                  [future pushOutput → xterm]
T8  SSH I/O thread: output_ready=true → emit()   [Rust emits to Tauri]
T9  Tauri → JS listener → pushOutput → xterm.write()
────────────────────────────────────────────────────────────────────────────
```

### Components

| Component | Role |
|-----------|------|
| `src-tauri/src/ssh/session.rs` | `SshSession.output_buffer` (VecDeque), `output_ready` (AtomicBool), `flush_output()` |
| `src-tauri/src/commands/terminal.rs` | `flush_session_output` Tauri command |
| `src/services/ipc.ts` | `flushSessionOutput()` wrapper |
| `src/services/outputBuffer.ts` | JS-side buffer: `initBuffer`, `pushOutput`, `flushBuffer`, `setWriter` |
| `src/pages/Connections.tsx` | listen → flush → navigate sequence |
| `src/hooks/useTerminal.ts` | Opens xterm, onReady flushes + sets writer |

### Resize Handling

Terminal sizing uses `window.addEventListener('resize', ...)` instead of `ResizeObserver`. This prevents infinite loops where xterm re-renders (on data arrival) cause ResizeObserver to fire, which calls `fit()`, which causes re-render, and so on.

### Logging

Rust-side tracing writes to `logs/hermes.log.{date}` (rolling daily). The `WorkerGuard` from `tracing_appender` is stored in `AppState._logging_guard` to keep it alive for the program's duration.

```
log dir: {working_dir}/logs/hermes.log.{YYYY-MM-DD}
filter:  hermes_remote_manager=debug,tauri=warn
format:  compact, no ANSI, file-only
```

### Event Naming Convention

All Tauri events use `terminal:{event}-{sessionId}` pattern:
- `terminal:connected-{id}` — SSH connection established
- `terminal:output-{id}` — Data from SSH channel stdout
- `terminal:stderr-{id}` — Data from SSH channel stderr
- `terminal:error-{id}` — I/O error
- `terminal:exit-{id}` — SSH session closed

### State Management

- **Rust**: `AppState` holds `Mutex<SessionManager>` for SSH sessions, `Mutex<Vault>` for credentials, `Mutex<SettingsManager>`, `Mutex<Connection>` (DB)
- **Frontend**: Zustand stores for `connections`, `folders`, `credentials`, `ui`, `session`

### IPC Naming

Rust structs for IPC use `#[serde(rename_all = "camelCase")]` so JSON keys match TypeScript interfaces. Flat Tauri command parameters auto-convert snake_case → camelCase in Tauri v2.

## Thread Safety

- SSH I/O runs on detached `std::thread` per session
- `Arc<Mutex<SessionState>>` shared between main thread and I/O thread
- `Arc<AtomicBool>` for `running` flag (lock-free)
- `Arc<Mutex<VecDeque<String>>>` for output buffer
- `Arc<AtomicBool>` for `output_ready` flag
- All Rust `emit()` calls use `let _ =` to ignore event system errors

## Database Schema (SQLite)

Tables: `connections`, `folders`, `credentials`, `settings`. Migrations auto-run via `src-tauri/src/storage/migrations/`. Location: platform app data directory.