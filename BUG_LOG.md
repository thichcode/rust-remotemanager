# Bug Log & Fixes

## 1. Rust struct snake_case vs frontend camelCase mismatch

**Error:** `invalid args 'req' for command 'create_connection': missing field 'authType'`  
**Root cause:** Rust model structs (`Connection`, `ConnectionCreateRequest`, `Credential`, `Folder`, `SshConfig`) used default serde (snake_case field names). Frontend types use camelCase (`credentialId`, `authType`, `keyPath`). The `createConnection` IPC call used `preparePayload()` to convert camelCase → snake_case before sending. But the RESPONSE from Rust (e.g., `list_connections`) also returned snake_case keys, so the Zustand store stored `credential_id` instead of `credentialId`. When `connectSSH` read `connection.credentialId`, it was `undefined` → credential resolution failed silently.

**Fix:**
- Added `#[serde(rename_all = "camelCase")]` to `Connection`, `ConnectionCreateRequest`, `Credential`, `Folder` in `src-tauri/src/storage/models.rs`
- Added `#[serde(rename_all = "camelCase")]` to `SshConfig` in `src-tauri/src/ssh/session.rs`
- Removed `preparePayload()` from `createConnection`, `updateConnection`, `createFolder`, `updateFolder` in `src/services/ipc.ts`
- Fixed `connectSSH` in `src/services/ipc.ts` to send camelCase keys directly (removed explicit snake_case conversion)
- Fixed `resolve_credential` in `src-tauri/src/commands/terminal.rs` to look for `credentialId` / `keyPath` (camelCase)

## 2. SSH credential resolution not triggered

**Error:** SSH connection with credential_id fails silently, connection proceeds without password/key.  
**Root cause:** `resolve_credential()` in `src-tauri/src/commands/terminal.rs` looked for `credential_id` in the config JSON, but the frontend sent `credentialId`. The function returned `Ok(())` without resolving when `credential_id` wasn't found.

**Fix:** Changed `config.get("credential_id")` → `config.get("credentialId")` and `config.insert("key_path"` → `config.insert("keyPath"` in `src-tauri/src/commands/terminal.rs`.

## 3. Event race condition — terminal output lost before listener registration

**Error:** After SSH connects successfully, terminal shows blank (black) screen.  
**Root cause:** The Rust I/O thread emits `terminal:output-{id}` events immediately after SSH connection succeeds. The frontend's output listener was registered inside `useTerminal.ts` (child component `useEffect`), which runs AFTER `TerminalPage.tsx`'s `useEffect`. Any output emitted between SSH connect and listener registration is permanently lost.

**Timeline:**
1. `connectSSH` returns session_id → Frontend navigates to `/terminal/{id}`
2. Rust I/O thread connects SSH → emits `terminal:connected` + starts emitting `terminal:output`
3. `TerminalPage` mounts → registers `connected`/`error`/`exit` listeners (in `useEffect`)
4. `TerminalSessionComponent` mounts → `useTerminal` effect runs → registers `output` listener too late

**Fix:**
- Moved `terminal:output` listener registration INTO `TerminalPage.tsx`'s `useEffect` (runs early, in parent component)
- Added an `outputBufferRef: string[]` that buffers ALL output events as they arrive
- `useTerminal.ts` receives the buffer ref, flushes buffered data when xterm is ready (after 50ms fit timeout)
- After flushing, overrides `Array.push` on the buffer to directly write to xterm (so future output goes straight to terminal)
- Removed the duplicate output listener from `useTerminal.ts` (now only registered in `TerminalPage.tsx`)

## 4. vaultStatus return type mismatch

**Error:** Frontend expected `{locked: bool}` but Rust returns plain `bool`.  
**Root cause:** Rust `vault_status` returns `bool` directly. Frontend type check against `result.data.locked` failed.  
**Fix:** Frontend now reads `result.data` directly (not `result.data.locked`).

## 5. getSettings return type mismatch

**Error:** Frontend expected `Setting[]` array but Rust returns `HashMap<String, String>`.  
**Root cause:** Rust's `get_settings` returns a serde_json Map, not a Vec of Setting structs.  
**Fix:** Frontend now handles the HashMap format.

## 6. Lock ordering in vault_unlock

**Error:** Potential deadlock when acquiring `db` then `vault` locks in inconsistent order.  
**Root cause:** `vault_unlock` locked `db` then `vault`, while other code locked `vault` then `db`.  
**Fix:** Changed order in `vault_unlock` to acquire `db` first, then `vault` (consistent with rest of codebase).

## 7. has_updates not reset per iteration

**Error:** After finding no updates, `has_updates` stayed `true` from previous iteration.  
**Root cause:** `has_updates` boolean was not reset at the start of each check cycle.  
**Fix:** Reset `has_updates = false` at top of each iteration in `src-tauri/src/commands/app.rs`.

## 8. keepalive_send not called in I/O loop

**Error:** SSH keepalive never sent despite `keepalive_interval` being set.  
**Root cause:** The keepalive_send() call was missing from the main I/O loop.  
**Fix:** Added periodic `session.keepalive_send()` call every ~100 iterations (1 second) in `src-tauri/src/ssh/session.rs`.

## 9. App.tsx used window.location.href instead of useNavigate

**Error:** Navigation breaks Tauri's webview context (full page reload instead of SPA navigation).  
**Root cause:** `window.location.href = ...` was used for routing instead of React Router's `useNavigate`.  
**Fix:** Replaced with `navigate()` from `react-router-dom`.

## 10. Silent salt corruption

**Error:** Vault salt silently corrupted without warning.  
**Root cause:** No validation or logging when salt read from file didn't match expected format.  
**Fix:** Added warning log when salt doesn't match expected length.

## 11. Terminal output race — refined fix

**Error:** Terminal still black after SSH connect (xterm doesn't display SSH output).  
**Root cause (Phase 1):** JS-side `listen()` promise resolves synchronously but the Tauri event registration requires an async IPC roundtrip. The `.then()` callback was triggered before the backend actually started forwarding events.

**Fix (Rust-side buffering):**
- Added `output_buffer: Arc<Mutex<VecDeque<String>>>` and `output_ready: Arc<AtomicBool>` to `SshSession` in `src-tauri/src/ssh/session.rs`
- I/O thread buffers output when `output_ready=false`, emits directly when `output_ready=true`
- `SessionManager::flush_output()` sets `output_ready=true` and returns buffered data
- New `flush_session_output` Tauri command exposed in `terminal.rs`
- Frontend calls `listenToTerminalOutput` → awaits its promise → then calls `flushSessionOutput` → then navigates
- This guarantees the listen registration completes BEFORE `output_ready` is set

**Files changed:**
- `src-tauri/src/ssh/session.rs`: buffer + output_ready + flush_output
- `src-tauri/src/commands/terminal.rs`: flush_session_output command
- `src/services/ipc.ts`: flushSessionOutput wrapper
- `src/pages/Connections.tsx`: listen → flush → navigate order
- `src/services/outputBuffer.ts`: module-level buffer with setWriter/flush
- `src/hooks/useTerminal.ts`: uses flushBuffer/setWriter/clearWriter

## 12. ResizeObserver infinite loop

**Error:** Typing in terminal triggers infinite resize loop, freezing the app.  
**Root cause:** xterm output → pixel-level layout shift → ResizeObserver fires → `fit()` called → terminal re-renders → ResizeObserver fires again → loop.  
**Fix:** 100ms debounce on ResizeObserver callback in `src/hooks/useTerminal.ts`.

## 13. File logging for debugging

**Error:** No Rust-side log visibility in release builds.  
**Fix:** Added `tracing-appender` crate + rolling daily log file at `logs/hermes.log.{date}` in `src-tauri/src/lib.rs`. Guard is kept alive in `AppState._logging_guard`.

## 14. DevTools console hidden in release

**Error:** Release builds use `windows_subsystem = "windows"`, hiding console.  
**Fix:** For `npm run tauri dev` (debug build), console window is visible. For `npm run tauri build` (release), use `tauri dev` to see Rust + JS console output simultaneously.

### Naming Convention Rules
- **Rust structs** for IPC: use `#[serde(rename_all = "camelCase")]` so JSON matches frontend.
- **Rust flat command params**: Tauri v2 auto-converts snake_case → camelCase. Frontend sends camelCase keys.
- **Rust struct command params**: Deserialized via struct's serde attributes. Use `rename_all = "camelCase"` consistently.
- **Frontend TypeScript interfaces**: Always use camelCase.

### Event Listener Registration Order
- Register Tauri event listeners as EARLY as possible (preferably in the first `useEffect` of the page component).
- Buffer events that arrive before the display component is ready.
- Flush buffer when display is ready; after flush, forward directly to display.

## Quick Reference — Build & Test Commands
```powershell
# Full build
cd src-tauri && cargo build
cd .. && npm run build
npm run tauri build

# Just typecheck (no build output)
npx tsc --noEmit

# Run tests
cd src-tauri && cargo test

# Dev mode
npm run tauri dev
```
