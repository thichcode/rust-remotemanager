# Testing Guide

## Overview

Hermes Remote Manager uses a layered test strategy: fast unit/contract tests for every commit, Rust backend tests for repositories and commands, and E2E tests (planned) for full workflows.

## Quick Commands

```bash
# Frontend unit tests (TypeScript)
npm run test

# Rust unit tests
npm run test:rust

# All tests
npm run test:all

# Full CI validation
npm run build && npm run test && npm run test:rust

# Quick CI (4 steps)
powershell -ExecutionPolicy Bypass -File scripts/ci-quick-test.ps1
```

## Test Structure

```
test/
  frontend-unit.test.ts    # Aggregator entrypoint
  types.test.ts             # Type enum contract tests
  stores.test.ts            # Zustand store tests
  output-buffer.test.ts     # Output buffer service tests
  ipc-contract.test.ts      # IPC wrapper contract tests
  support/
    types.ts                # Test-only types (MockInvokeCall, etc.)
    testRunner.ts           # Lightweight test runner
    fixtures.ts             # Canonical fixtures (makeConnection, etc.)
    mockTauri.ts            # Mock invoke/listen with call recording

src-tauri/src/
  test_support.rs           # Rust test helpers (TestDatabase, builders)
```

## Frontend Tests

### Types Contract (`test/types.test.ts`)
Verifies TypeScript enum values match backend contract.

```bash
npx tsx test/types.test.ts
```

### Store Tests (`test/stores.test.ts`)
Tests Zustand stores: `useConnectionStore`, `useSessionStore`, `useUiStore`.

```bash
npx tsx test/stores.test.ts
```

### Output Buffer Tests (`test/output-buffer.test.ts`)
Tests the `outputBuffer.ts` service for buffer, flush, writer, and cleanup.

```bash
npx tsx test/output-buffer.test.ts
```

### IPC Contract Tests (`test/ipc-contract.test.ts`)
Tests all `src/services/ipc.ts` wrappers: command names, payload shapes, event listeners, success/error handling. Uses `mockTauri.ts` to mock Tauri `invoke`/`listen` without real backend.

```bash
npx tsx test/ipc-contract.test.ts
```

## Rust Tests

All Rust tests run via `cargo test`. Coverage includes:
- Commands: connections, folders, credentials, vault, settings, terminal, SFTP, tunnels, RDP
- Storage: repositories (connection, credential, folder), database
- Security: vault, crypto
- SSH: session manager

```bash
cd src-tauri
cargo test
```

### Using test_support.rs

```rust
#[cfg(test)]
mod tests {
    use crate::test_support::*;

    #[test]
    fn test_with_fixture() {
        let db = TestDatabase::new();
        let req = connection_request("test-server");
        // use db.connection() for repo tests
    }

    #[test]
    fn test_connection_builder() {
        let req = ssh_connection("prod", "192.168.1.10", "admin");
        assert_eq!(req.host, "192.168.1.10");
        assert_eq!(req.auth_type, "password");
    }
}
```

## CI Scripts

| Script | Purpose |
|--------|---------|
| `scripts/ci-quick-test.ps1` | Fast 4-step: build + test + rust + tauri-build |
| `scripts/test-app.ps1` | Comprehensive: phases, log analysis, git status |
| `scripts/parse-logs.ps1` | Parse Rust logs for errors/warnings |

## Planned: E2E Tests

E2E testing requires Docker + WebDriverIO. See `implementation_plan.md` for full E2E specification. Planned:
- `e2e/specs/app-smoke.e2e.ts` — startup, navigation, persistence
- `e2e/specs/connections.e2e.ts` — connection CRUD, search, favorites
- `e2e/specs/vault-credentials.e2e.ts` — vault unlock, credential management
- `e2e/specs/ssh-terminal.e2e.ts` — SSH connect, output, command execution
- `e2e/specs/sftp.e2e.ts` — list/upload/download/mkdir/rename/delete
- `e2e/specs/tunnels.e2e.ts` — tunnel create/list/stop
- `e2e/specs/settings.e2e.ts` — theme, settings persistence

Docker test server: `e2e/docker/ssh-server/` (Dockerfile + docker-compose.yml)

## Environment Variables

No required environment variables for unit tests. E2E tests will use:
- `E2E_REMOTE_HOST` — SSH target host (default: localhost)
- `E2E_SSH_PORT` — SSH port (default: 22)
- `E2E_USERNAME` — SSH username
- `E2E_PASSWORD` — SSH password (or `E2E_PRIVATE_KEY_PATH`)
- `E2E_APP_DATA_DIR` — isolated app data directory for E2E runs

## Troubleshooting

### Frontend tests fail with "window is not defined"
IPC wrappers call `log_json()` which uses `window`. This is expected in Node test environment — logging errors are non-fatal and do not affect test results.

### Rust tests hang
Some SSH session tests may timeout if no real SSH target is available. Tests that require network should be marked with `#[ignore]`.

### Test data isolation
Each `TestDatabase` creates a fresh temporary directory. Rust tests are independent. Frontend tests use mocked IPC — no real database needed.

## Adding New Tests

### Frontend (TypeScript)

1. Add fixture in `test/support/fixtures.ts` if needed
2. Add test file with `test()` from `test/support/testRunner.ts`
3. Import from `test/support/fixtures.ts` for fixtures
4. Use `installMockTauri()` / `resetMockTauri()` for IPC mocking
5. Add to `test/frontend-unit.test.ts` aggregator

### Rust

1. Add helper in `src-tauri/src/test_support.rs` if needed
2. Add `#[cfg(test)]` module or inline `#[test]` functions
3. Use `TestDatabase` for repo integration tests
4. Use `connection_request()` or `TestConnectionRequestBuilder` for fixture creation

## Success Criteria

- All `npm run test` tests pass
- All `npm run test:rust` tests pass
- `npm run build` succeeds
- CI script passes without errors