import assert from 'node:assert/strict';
import { useSessionStore } from '../src/stores/sessionStore';
import { useConnectionStore } from '../src/stores/connectionStore';
import { useUiStore } from '../src/stores/uiStore';
import {
  initBuffer,
  pushOutput,
  flushBuffer,
  setWriter,
  clearWriter,
  cleanupBuffer,
} from '../src/services/outputBuffer';
import {
  AuthType,
  ConnectionType,
  ProxyType,
  TunnelType,
  type Connection,
  type Folder,
  type TerminalSession,
} from '../src/services/types';

type TestCase = {
  name: string;
  run: () => void | Promise<void>;
};

const tests: TestCase[] = [];

function test(name: string, run: TestCase['run']) {
  tests.push({ name, run });
}

function session(id: string, overrides: Partial<TerminalSession> = {}): TerminalSession {
  return {
    id,
    connectionId: `conn-${id}`,
    state: 'connecting',
    createdAt: new Date().toISOString(),
    ...overrides,
  };
}

function connection(id: string, overrides: Partial<Connection> = {}): Connection {
  return {
    id,
    name: `Server ${id}`,
    type: ConnectionType.SSH,
    host: '127.0.0.1',
    port: 22,
    username: 'admin',
    authType: AuthType.Password,
    sortOrder: 0,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    ...overrides,
  };
}

function folder(id: string, overrides: Partial<Folder> = {}): Folder {
  return {
    id,
    name: `Folder ${id}`,
    sortOrder: 0,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    ...overrides,
  };
}

function resetStores() {
  useSessionStore.getState().clearSessions();
  useConnectionStore.getState().setConnections([]);
  useConnectionStore.getState().setFolders([]);
  useConnectionStore.getState().setSelectedConnectionId(null);
  useConnectionStore.getState().setSearchTerm('');
  useConnectionStore.getState().setFilterType('all');
  useUiStore.getState().setSidebarCollapsed(false);
  useUiStore.getState().closeConnectionForm();
  useUiStore.getState().closeFolderDialog();
  useUiStore.getState().setShowSettings(false);
}

// Session store
test('sessionStore: add/remove/update/clear sessions', () => {
  resetStores();
  useSessionStore.getState().addSession(session('s1'));
  useSessionStore.getState().addSession(session('s2'));
  assert.equal(useSessionStore.getState().sessions.length, 2);
  assert.equal(useSessionStore.getState().activeSessionId, 's2');

  useSessionStore.getState().setActiveSession('s1');
  assert.equal(useSessionStore.getState().activeSessionId, 's1');

  useSessionStore.getState().updateSessionState('s1', 'connected');
  assert.equal(useSessionStore.getState().sessions.find((s) => s.id === 's1')?.state, 'connected');

  useSessionStore.getState().removeSession('s2');
  assert.equal(useSessionStore.getState().activeSessionId, 's1');

  useSessionStore.getState().clearSessions();
  assert.deepEqual(useSessionStore.getState().sessions, []);
  assert.equal(useSessionStore.getState().activeSessionId, null);
});

// Connection store
test('connectionStore: connections/folders/filter/search operations', () => {
  resetStores();
  useConnectionStore.getState().addConnection(connection('c1', { name: 'Old' }));
  useConnectionStore.getState().updateConnection(connection('c1', { name: 'New' }));
  assert.equal(useConnectionStore.getState().connections[0].name, 'New');

  useConnectionStore.getState().setSelectedConnectionId('c1');
  useConnectionStore.getState().removeConnection('c1');
  assert.equal(useConnectionStore.getState().selectedConnectionId, null);

  useConnectionStore.getState().addFolder(folder('f1', { name: 'Old Folder' }));
  useConnectionStore.getState().updateFolder(folder('f1', { name: 'New Folder' }));
  assert.equal(useConnectionStore.getState().folders[0].name, 'New Folder');
  useConnectionStore.getState().removeFolder('f1');
  assert.equal(useConnectionStore.getState().folders.length, 0);

  useConnectionStore.getState().setSearchTerm('prod');
  useConnectionStore.getState().setFilterType(ConnectionType.RDP);
  assert.equal(useConnectionStore.getState().searchTerm, 'prod');
  assert.equal(useConnectionStore.getState().filterType, ConnectionType.RDP);
});

// UI store
test('uiStore: dialog/sidebar/settings operations', () => {
  resetStores();
  useUiStore.getState().toggleSidebar();
  assert.equal(useUiStore.getState().sidebarCollapsed, true);

  useUiStore.getState().openConnectionForm(connection('c1'));
  assert.equal(useUiStore.getState().showConnectionForm, true);
  assert.equal(useUiStore.getState().editingConnection?.id, 'c1');
  useUiStore.getState().closeConnectionForm();
  assert.equal(useUiStore.getState().editingConnection, null);

  useUiStore.getState().openFolderDialog(folder('f1'));
  assert.equal(useUiStore.getState().showFolderDialog, true);
  assert.equal(useUiStore.getState().editingFolder?.id, 'f1');
  useUiStore.getState().closeFolderDialog();
  assert.equal(useUiStore.getState().editingFolder, null);

  useUiStore.getState().setShowSettings(true);
  assert.equal(useUiStore.getState().showSettings, true);
});

// Output buffer
test('outputBuffer: buffer, flush, direct writer, cleanup', () => {
  cleanupBuffer('s1');
  const writes: string[] = [];

  initBuffer('s1');
  pushOutput('s1', 'a');
  pushOutput('s1', 'b');
  flushBuffer('s1', (data) => writes.push(data));
  assert.deepEqual(writes, ['a', 'b']);

  setWriter('s1', (data) => writes.push(data));
  pushOutput('s1', 'c');
  assert.deepEqual(writes, ['a', 'b', 'c']);

  clearWriter('s1');
  pushOutput('s1', 'd');
  flushBuffer('s1', (data) => writes.push(data));
  assert.deepEqual(writes, ['a', 'b', 'c', 'd']);

  cleanupBuffer('s1');
  pushOutput('s1', 'ignored');
  assert.deepEqual(writes, ['a', 'b', 'c', 'd']);
});

// Types
test('types: enum values match backend contract', () => {
  assert.equal(ConnectionType.SSH, 'ssh');
  assert.equal(ConnectionType.RDP, 'rdp');
  assert.equal(ConnectionType.Serial, 'serial');
  assert.equal(AuthType.Password, 'password');
  assert.equal(AuthType.Key, 'key');
  assert.equal(AuthType.Agent, 'agent');
  assert.equal(ProxyType.None, 'none');
  assert.equal(ProxyType.Socks5, 'socks5');
  assert.equal(ProxyType.Http, 'http');
  assert.equal(TunnelType.Local, 'local');
  assert.equal(TunnelType.Remote, 'remote');
  assert.equal(TunnelType.Dynamic, 'dynamic');
});

async function main() {
  let passed = 0;
  for (const t of tests) {
    await t.run();
    passed += 1;
    console.log(`✓ ${t.name}`);
  }
  console.log(`\nFrontend unit tests: ${passed}/${tests.length} passed`);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});