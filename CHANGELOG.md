# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- **File logging** — Rust backend now writes rolling daily logs to `logs/hermes.log.{date}` for debugging in release builds. Uses `tracing-appender` crate.
- **`flush_session_output` command** — Rust-side output buffering to fix terminal race condition. Terminal output is buffered until the frontend explicitly calls flush, eliminating the race window between SSH connect and event listener registration.

### Fixed
- **Terminal black screen** — SSH output now displays correctly in xterm. Root cause was event listener registration timing; fixed with Rust-side buffering + listen-then-flush-then-navigate sequence.
- **ResizeObserver infinite loop** — Terminal now handles typing without freezing. Added 100ms debounce on ResizeObserver callbacks in `useTerminal.ts`.
- **Credential resolution** — SSH connections with saved credentials now properly resolve passwords and key paths.
- **Lock ordering** — Fixed potential deadlock in `vault_unlock` by consistent lock ordering (db → vault).
- **keepalive_send** — SSH keepalive is now properly sent every ~1s in the I/O loop.
- **`has_updates` reset** — Fixed stale flag persisting across check cycles.
- **Naming conventions** — All Rust IPC structs now use `#[serde(rename_all = "camelCase")]` for consistent JSON serialization with frontend.

### Changed
- **Output buffering architecture** — Replaced JS-side `outputBufferRef` approach with Rust-side `VecDeque<String>` buffer. Frontend now: (1) registers listener, (2) awaits it, (3) flushes Rust buffer, (4) navigates. This guarantees zero output loss during navigation.

## [0.1.0] — 2026-05-19

### Added
- SSH terminal sessions with xterm.js
- Connection management (save, edit, delete, favorite)
- Folder organization for connections
- Credential management with encrypted storage
- Vault (encrypted credential store) with password protection
- RDP support (generate .rdp file + launch mstsc.exe)
- SFTP file browser
- SSH tunnel management
- Dark theme UI
- Terminal search (Ctrl+F)