import assert from 'node:assert/strict';
import { useConnectionStore } from '../src/stores/connectionStore';
import { useSessionStore } from '../src/stores/sessionStore';
import { useUiStore } from '../src/stores/uiStore';
import { ConnectionType } from '../src/services/types';
import { makeConnection, makeFolder, makeTerminalSession } from './support/fixtures';
import { failOnError, runTests, test } from './support/testRunner';

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

test('sessionStore: add/remove/update/clear sessions', () => {
  resetStores();
  useSessionStore.getState().addSession(makeTerminalSession('s1'));
  useSessionStore.getState().addSession(makeTerminalSession('s2'));
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

test('connectionStore: connections/folders/filter/search operations', () => {
  resetStores();
  useConnectionStore.getState().addConnection(makeConnection('c1', { name: 'Old' }));
  useConnectionStore.getState().updateConnection(makeConnection('c1', { name: 'New' }));
  assert.equal(useConnectionStore.getState().connections[0].name, 'New');

  useConnectionStore.getState().setSelectedConnectionId('c1');
  useConnectionStore.getState().removeConnection('c1');
  assert.equal(useConnectionStore.getState().selectedConnectionId, null);

  useConnectionStore.getState().addFolder(makeFolder('f1', { name: 'Old Folder' }));
  useConnectionStore.getState().updateFolder(makeFolder('f1', { name: 'New Folder' }));
  assert.equal(useConnectionStore.getState().folders[0].name, 'New Folder');
  useConnectionStore.getState().removeFolder('f1');
  assert.equal(useConnectionStore.getState().folders.length, 0);

  useConnectionStore.getState().setSearchTerm('prod');
  useConnectionStore.getState().setFilterType(ConnectionType.RDP);
  assert.equal(useConnectionStore.getState().searchTerm, 'prod');
  assert.equal(useConnectionStore.getState().filterType, ConnectionType.RDP);
});

test('uiStore: dialog/sidebar/settings operations', () => {
  resetStores();
  useUiStore.getState().toggleSidebar();
  assert.equal(useUiStore.getState().sidebarCollapsed, true);

  useUiStore.getState().openConnectionForm(makeConnection('c1'));
  assert.equal(useUiStore.getState().showConnectionForm, true);
  assert.equal(useUiStore.getState().editingConnection?.id, 'c1');
  useUiStore.getState().closeConnectionForm();
  assert.equal(useUiStore.getState().editingConnection, null);

  useUiStore.getState().openFolderDialog(makeFolder('f1'));
  assert.equal(useUiStore.getState().showFolderDialog, true);
  assert.equal(useUiStore.getState().editingFolder?.id, 'f1');
  useUiStore.getState().closeFolderDialog();
  assert.equal(useUiStore.getState().editingFolder, null);

  useUiStore.getState().setShowSettings(true);
  assert.equal(useUiStore.getState().showSettings, true);
});

runTests('Store tests').catch(failOnError);
