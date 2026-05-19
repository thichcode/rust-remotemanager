# Implementation Plan

[Overview]
Build full test automation for Hermes Remote Manager covering frontend unit tests, Rust backend tests, IPC contracts, Tauri UI E2E tests, and local SSH/SFTP/tunnel workflows.

Hermes Remote Manager is a Tauri 2 desktop application with a React 18/TypeScript frontend and a Rust backend. Existing automation includes `test/frontend-unit.test.ts`, many Rust inline `#[cfg(test)]` tests, and `scripts/test-app.ps1`, but coverage is not yet organized around all user-facing functions: connection CRUD, folders, vault/credentials, SSH terminal, SFTP browser, settings, IPC boundaries, and tunnels.

The implementation should introduce layered automation: fast deterministic unit/contract tests for every commit, Rust backend integration tests around repositories and commands, Tauri UI smoke tests, and full remote E2E tests using a local SSH/SFTP/tunnel target. The full suite should not require real remote infrastructure or real secrets; it should use isolated app data, deterministic local fixtures, disposable test credentials, and cleanup scripts.

[Types]
Add test-only TypeScript and Rust types for fixtures, mocked IPC calls, E2E configuration, and local remote-server metadata.

Create `test/support/types.ts` with `TestSuiteName`, `TestCase`, `MockInvokeCall`, `MockInvokeRule<T>`, `MockTauriEvent<T>`, and fixture-facing aliases for app models imported from `src/services/types.ts`. `MockInvokeCall` must contain the Tauri command `method` and optional `args`; `MockInvokeRule<T>` must support optional argument matching, success result, or simulated error.

Create `e2e/support/e2e-env.ts` with `RemoteTestServerConfig` and `TauriE2EConfig`. `RemoteTestServerConfig` must include `host`, `sshPort`, `username`, `password`, optional `privateKeyPath`, and `sftpRoot`. `TauriE2EConfig` must include isolated `appDataDir`, `webDriverUrl`, optional `appBinaryPath`, `remote`, `timeoutMs`, and optional `headless`.

Create Rust test support types in `src-tauri/src/test_support.rs` under `#[cfg(test)]`: `TestDatabase` wrapping a temp SQLite database and `TestConnectionRequestBuilder` for valid `ConnectionCreateRequest` fixtures. Validation rules: ports must be `1..=65535`, connection fixture `name`/`host`/`username` must be non-empty, test secrets must be local-only, and app data paths must be unique per run.

[Files]
Add a structured test hierarchy, update scripts/configuration, and keep production source changes limited to testability seams.

New files:
- `test/support/testRunner.ts`: shared lightweight TypeScript test runner.
- `test/support/fixtures.ts`: canonical fixtures for `Connection`, `ConnectionFormData`, `Folder`, `Credential`, `TerminalSession`, `SftpFileInfo`, and `TunnelConfig`.
- `test/support/mockTauri.ts`: mock `invoke` and `listen` support with call recording, event simulation, and reset/restore.
- `test/support/types.ts`: shared test-only TypeScript types.
- `test/ipc-contract.test.ts`: IPC wrapper contract tests for `src/services/ipc.ts`.
- `test/stores.test.ts`: focused Zustand store tests.
- `test/output-buffer.test.ts`: focused `src/services/outputBuffer.ts` tests.
- `test/connection-form-validation.test.ts`: validation tests if `ConnectionForm` helpers are extracted.
- `test/sftp-utils.test.ts`: utility tests if SFTP helpers are extracted.
- `e2e/support/e2e-env.ts`, `e2e/support/tauriDriver.ts`, `e2e/support/remoteServer.ts`: E2E environment, driver, and remote target helpers.
- `e2e/specs/app-smoke.e2e.ts`: startup/navigation/basic persistence smoke test.
- `e2e/specs/connections.e2e.ts`: connection/folder CRUD, edit, favorite, search, delete, restart persistence.
- `e2e/specs/vault-credentials.e2e.ts`: vault unlock/lock and local test credential creation/selection.
- `e2e/specs/ssh-terminal.e2e.ts`: local SSH connection, output buffering, command input, resize, disconnect.
- `e2e/specs/sftp.e2e.ts`: list/upload/download/mkdir/rename/delete using active SSH session.
- `e2e/specs/tunnels.e2e.ts`: create/list/stop tunnel and, where possible, validate forwarded traffic against a deterministic local service.
- `e2e/specs/settings.e2e.ts`: settings/theme/vault UI behavior.
- `e2e/docker/ssh-server/Dockerfile` and `e2e/docker/docker-compose.yml`: deterministic local SSH/SFTP/tunnel target.
- `scripts/e2e.ps1`, `scripts/e2e.sh`, `scripts/ci-full-test.ps1`: orchestration and CI entrypoints.
- `docs/TESTING.md`: test architecture, commands, prerequisites, environment variables, CI strategy, and troubleshooting.
- `src-tauri/src/test_support.rs`: Rust test helpers.

Existing files to modify:
- `package.json`: add scripts `test:frontend`, `test:ipc`, `test:unit`, `test:e2e:smoke`, `test:e2e:remote`, `test:e2e`, `test:ci`, and `test:full`; add local dev dependencies for chosen E2E/mocking tools.
- `tsconfig.json` or new `tsconfig.test.json`: include test/e2e TypeScript without weakening production strictness.
- `test/frontend-unit.test.ts`: convert to an aggregator or refactor to use shared runner/fixtures while preserving `npm run test` compatibility.
- `scripts/test-app.ps1`: keep as legacy smoke script or delegate to new split/full scripts.
- `src/services/ipc.ts`: expose/refactor payload helpers only if necessary; otherwise test through public wrappers.
- `src/components/connections/ConnectionForm.tsx`: optionally extract validation/normalization into `connectionFormModel.ts`.
- `src/components/terminal/SftpBrowser.tsx`: optionally extract helpers into `sftpUtils.ts`.
- `src-tauri/src/lib.rs`: add only `#[cfg(test)] mod test_support;`.
- `src-tauri/src/**`: add/refactor tests using `test_support` where practical.
- `.gitignore`: ignore `.e2e-artifacts/`, `.test-appdata/`, generated SSH keys, screenshots, traces, and temp Docker data.
- `README.md`: add a short Testing section linking to `docs/TESTING.md`.

Files to delete or move: no production files should be deleted. Move duplicated frontend test fixtures/runner logic out of `test/frontend-unit.test.ts`. Remove temporary planning artifacts `.plan_tmp_structure.txt`, `.plan_tmp_defs.txt`, `.plan_tmp_imports.txt`, `.plan_tmp_manifests.txt`, and `.plan_tmp_todos.txt` before completion.

[Functions]
Add shared runners, fixture builders, IPC mocks, E2E helpers, optional pure UI helpers, and Rust backend test helpers.

New TypeScript functions:
- `test/support/testRunner.ts::test(name, run)`, `runTests(tests?)`, and `resetTests()`.
- `test/support/fixtures.ts::makeConnection`, `makeConnectionFormData`, `makeFolder`, `makeCredential`, `makeTerminalSession`, `makeSftpFileInfo`, and `makeTunnelConfig`.
- `test/support/mockTauri.ts::installMockTauri`, `mockInvoke`, `mockInvokeError`, `emitTauriEvent`, `getInvokeCalls`, and `resetMockTauri`.
- `e2e/support/e2e-env.ts::loadE2EConfig()`.
- `e2e/support/remoteServer.ts::startRemoteTestServer(config)` and `stopRemoteTestServer(handle)`.
- `e2e/support/tauriDriver.ts::startTauriApp(config)`, `stopTauriApp(driver)`, `waitForText(driver, text, timeoutMs?)`, and `captureFailureArtifacts(driver, testName)`.

Optional extracted production-helper functions:
- `src/components/connections/connectionFormModel.ts::validateConnectionForm(data, options?)`, `normalizeStartupCommands(input)`, and `connectionToFormData(connection)`.
- `src/components/terminal/sftpUtils.ts::formatFileSize(bytes)`, `formatPermissions(mode)`, `getParentPath(path)`, and `joinPath(base, name)`.

New Rust functions:
- `src-tauri/src/test_support.rs::create_test_database() -> TestDatabase`.
- `src-tauri/src/test_support.rs::connection_request(name: &str) -> ConnectionCreateRequest`.
- `src-tauri/src/test_support.rs::credential_fixture(...) -> Credential` if needed.
- Optional assertion helper for `AppError` messages.

Modified functions:
- `test/frontend-unit.test.ts::main()` should use `runTests()` or become an aggregate entrypoint.
- `ConnectionForm` validation/edit/startup mapping should delegate to extracted helpers if created.
- `SftpBrowser` path/format helpers should delegate to extracted helpers if created.
- `scripts/test-app.ps1` should call the new commands or remain documented as legacy.

Removed functions: no production functions should be removed; remove duplicated inline frontend test fixture functions after shared fixtures exist.

[Classes]
No production classes need to be added; introduce only test support controllers/builders.

New class-like controllers:
- `MockTauriController` in `test/support/mockTauri.ts` with methods for mock rule registration, event emission, call inspection, reset, and restore.
- `TauriDriver` in `e2e/support/tauriDriver.ts` wrapping WebDriver actions such as click-by-text, type, wait, screenshot, close, and restart with same app data.
- `RemoteTestServerHandle` in `e2e/support/remoteServer.ts` containing container IDs, mapped ports, credentials, and cleanup callbacks.
- `TestConnectionRequestBuilder` in `src-tauri/src/test_support.rs` for valid Rust connection request fixtures.

Modified classes: `ErrorBoundary` should not require production changes, but E2E smoke tests should verify its fallback does not appear. `AppState` should not change except if an explicitly test-only app builder becomes necessary.

Removed classes: none.

[Dependencies]
Add local E2E automation and optional mocking dependencies while keeping unit tests lightweight.

Recommended Node dev dependencies: `webdriverio`, `@wdio/cli`, `@wdio/local-runner`, `@wdio/mocha-framework`, `@wdio/spec-reporter`, optionally `mocha`, optionally `esmock` if ESM Tauri API mocking cannot be implemented cleanly, and optionally `wait-on` for service readiness.

Recommended Rust dev dependencies: keep existing `tempfile = "3"`; add `serial_test` only if fixed ports/global state need serialized tests.

External tooling: Docker or Docker Desktop for full remote E2E, and Tauri-compatible WebDriver. Scripts must detect unavailable Docker/WebDriver and either fail clearly or skip only when documented by environment flags. No global npm packages should be required.

Package scripts should include:
```json
{
  "test:frontend": "tsx test/frontend-unit.test.ts",
  "test:ipc": "tsx test/ipc-contract.test.ts",
  "test:unit": "npm run test:frontend && npm run test:ipc",
  "test:rust": "cargo test --manifest-path src-tauri/Cargo.toml",
  "test:e2e:smoke": "wdio run e2e/wdio.conf.ts --suite smoke",
  "test:e2e:remote": "wdio run e2e/wdio.conf.ts --suite remote",
  "test:e2e": "powershell -ExecutionPolicy Bypass -File scripts/e2e.ps1",
  "test:ci": "npm run build && npm run test:unit && npm run test:rust",
  "test:full": "powershell -ExecutionPolicy Bypass -File scripts/ci-full-test.ps1"
}
```

[Testing]
Validate the automation by running quick unit/contract tests, Rust tests, build checks, smoke E2E, and full remote E2E against a local SSH/SFTP/tunnel server.

Frontend coverage requirements: stores, output buffer, enum/backend contract values, IPC wrapper command names and payload shapes, terminal event listener names, SFTP wrappers, vault/settings wrappers, tunnel payload conversion, and extracted helper behavior where applicable.

Rust coverage requirements: existing tests must pass; add missing repository tests for tags/notes/startup commands/keepalive/favorite persistence; add command tests for validation/not-found behavior; add vault lock/unlock/plaintext re-encryption coverage; add settings persistence coverage; add SFTP error-path and tunnel manager tests where possible without live network dependency.

E2E smoke requirements: launch with isolated app data, ensure no ErrorBoundary fallback, navigate Dashboard/Connections/Settings/Terminal empty state, create folder/connection profile, restart with same app data, and verify persistence.

Full remote E2E requirements: start local SSH/SFTP server, unlock vault, save test credential, create SSH connection, connect, verify terminal output and command execution (`echo hermes-e2e`, `pwd`), resize and disconnect, perform SFTP list/upload/download/mkdir/rename/delete, create/list/stop tunnels, and validate forwarded traffic if the Docker environment includes a deterministic service.

Validation commands:
```powershell
npm run test:unit
npm run test:rust
npm run build
npm run test:ci
npm run test:e2e:smoke
npm run test:e2e
```

Success criteria: all unit/Rust/build checks pass, E2E smoke uses isolated app data and cleans up, full E2E can provision or validate the local remote target and clean it up, and `docs/TESTING.md` documents prerequisites, commands, environment variables, CI usage, and troubleshooting.

[Implementation Order]
Implement in layers: shared test foundations, fast tests, backend helpers, E2E infrastructure, remote scenarios, then documentation and CI scripts.

1. Remove temporary planning artifacts if present.
2. Create `test/support/types.ts`, `test/support/testRunner.ts`, `test/support/fixtures.ts`, and `test/support/mockTauri.ts`.
3. Refactor `test/frontend-unit.test.ts` and add/split `test/stores.test.ts`, `test/output-buffer.test.ts`, and type contract coverage.
4. Add `test/ipc-contract.test.ts` for all `src/services/ipc.ts` public wrappers.
5. Optionally extract and test `connectionFormModel.ts` and `sftpUtils.ts`.
6. Add `src-tauri/src/test_support.rs`, register it in `src-tauri/src/lib.rs`, and add/refactor Rust tests.
7. Update `package.json`, test TypeScript configuration, and local dev dependencies.
8. Create E2E support files for config, Tauri driver, artifacts, and remote server orchestration.
9. Create Docker SSH/SFTP/tunnel test server files.
10. Add E2E specs for smoke, connections, vault/credentials, SSH terminal, SFTP, tunnels, and settings.
11. Add `scripts/e2e.ps1`, `scripts/e2e.sh`, and `scripts/ci-full-test.ps1`.
12. Update `.gitignore`, `README.md`, and create `docs/TESTING.md`.
13. Run validation commands in order: `npm run test:unit`, `npm run test:rust`, `npm run build`, `npm run test:ci`, `npm run test:e2e:smoke`, `npm run test:e2e`.
14. Fix failures without weakening production validation or silently skipping required full-functionality scenarios.