import assert from 'node:assert/strict';
import { AuthType } from '../src/services/types';
import {
  connectRDP,
  connectSSH,
  createConnection,
  createTunnel,
  deleteConnection,
  deleteCredential,
  deleteFolder,
  disconnectSession,
  flushSessionOutput,
  getCredentials,
  getSessionState,
  getSettings,
  listenToTerminalConnected,
  listenToTerminalError,
  listenToTerminalExit,
  listenToTerminalOutput,
  listConnections,
  listFolders,
  listSftpDir,
  listTunnels,
  pickSSHKeyFile,
  saveCredential,
  sftpDownload,
  sftpMkdir,
  sftpRename,
  sftpRm,
  sftpStat,
  sftpUpload,
  stopTunnel,
  terminalInput,
  terminalResize,
  updateConnection,
  updateFolder,
  updateSetting,
  vaultLock,
  vaultStatus,
  vaultUnlock,
} from '../src/services/ipc';
import { makeConnection, makeConnectionFormData, makeFolder, makeSftpFileInfo, makeTunnelConfig } from './support/fixtures';
import { emitTauriEvent, getInvokeCalls, installMockTauri, mockInvoke, mockInvokeError, resetMockTauri, restoreMockTauri } from './support/mockTauri';
import { failOnError, runTests, test } from './support/testRunner';

function lastCall() {
  const calls = getInvokeCalls();
  return calls[calls.length - 1];
}

installMockTauri();

test('ipc: wraps success and errors consistently', async () => {
  resetMockTauri();
  mockInvoke('list_connections', [makeConnection('c1')]);
  const ok = await listConnections();
  assert.equal(ok.success, true);
  assert.equal(ok.data?.[0].id, 'c1');
  assert.equal(lastCall().method, 'list_connections');

  resetMockTauri();
  mockInvokeError('list_connections', 'boom');
  const failed = await listConnections();
  assert.equal(failed.success, false);
  assert.equal(failed.error, 'boom');
});

test('ipc: connection wrappers use expected commands and payloads', async () => {
  resetMockTauri();
  const form = makeConnectionFormData({ tags: ['prod', 'db'] });
  mockInvoke('create_connection', makeConnection('created'));
  await createConnection(form);
  assert.equal(lastCall().method, 'create_connection');
  assert.equal((lastCall().args?.req as Record<string, unknown>).tags, JSON.stringify(['prod', 'db']));

  resetMockTauri();
  const conn = makeConnection('c1', { startupCommands: ['pwd'], tags: ['prod'] });
  mockInvoke('update_connection', conn);
  await updateConnection(conn);
  assert.equal(lastCall().method, 'update_connection');
  assert.equal((lastCall().args?.conn as Record<string, unknown>).startupCommands, JSON.stringify(['pwd']));

  resetMockTauri();
  mockInvoke('delete_connection', undefined);
  await deleteConnection('c1');
  assert.deepEqual(lastCall(), { method: 'delete_connection', args: { id: 'c1' } });
});

test('ipc: folder wrappers use expected commands', async () => {
  resetMockTauri();
  mockInvoke('list_folders', [makeFolder('f1')]);
  await listFolders();
  assert.equal(lastCall().method, 'list_folders');

  resetMockTauri();
  mockInvoke('create_folder', makeFolder('f2'));
  await import('../src/services/ipc').then((m) => m.createFolder({ name: 'New', sortOrder: 1 }));
  assert.deepEqual(lastCall().args, { name: 'New', sortOrder: 1 });

  resetMockTauri();
  mockInvoke('update_folder', makeFolder('f1'));
  await updateFolder(makeFolder('f1'));
  assert.equal(lastCall().method, 'update_folder');

  resetMockTauri();
  mockInvoke('delete_folder', undefined);
  await deleteFolder('f1');
  assert.deepEqual(lastCall(), { method: 'delete_folder', args: { id: 'f1' } });
});

test('ipc: credential, vault, and settings wrappers use expected payloads', async () => {
  resetMockTauri();
  mockInvoke('list_credentials', []);
  await getCredentials();
  assert.equal(lastCall().method, 'list_credentials');

  resetMockTauri();
  mockInvoke('save_credential', { id: 'cred-1' });
  await saveCredential({ name: 'Key', authType: AuthType.Key, username: 'u', keyPath: 'C:/key' });
  assert.deepEqual(lastCall().args, {
    name: 'Key', authType: AuthType.Key, username: 'u', password: undefined,
    privateKey: undefined, keyPath: 'C:/key', passphraseProtected: undefined,
  });

  resetMockTauri();
  mockInvoke('delete_credential', undefined);
  await deleteCredential('cred-1');
  assert.deepEqual(lastCall(), { method: 'delete_credential', args: { id: 'cred-1' } });

  resetMockTauri();
  mockInvoke('pick_ssh_key_file', 'C:/key');
  await pickSSHKeyFile();
  assert.equal(lastCall().method, 'pick_ssh_key_file');

  resetMockTauri();
  mockInvoke('vault_status', true);
  await vaultStatus();
  assert.equal(lastCall().method, 'vault_status');

  resetMockTauri();
  mockInvoke('vault_unlock', undefined);
  await vaultUnlock('secret');
  assert.deepEqual(lastCall(), { method: 'vault_unlock', args: { password: 'secret' } });

  resetMockTauri();
  mockInvoke('vault_lock', undefined);
  await vaultLock();
  assert.equal(lastCall().method, 'vault_lock');

  resetMockTauri();
  mockInvoke('get_settings', { theme: 'dark' });
  await getSettings();
  assert.equal(lastCall().method, 'get_settings');

  resetMockTauri();
  mockInvoke('update_setting', undefined);
  await updateSetting('theme', 'dark');
  assert.deepEqual(lastCall(), { method: 'update_setting', args: { key: 'theme', value: 'dark' } });
});

test('ipc: terminal and RDP wrappers use expected payloads', async () => {
  resetMockTauri();
  mockInvoke('connect_rdp', 'rdp-session');
  await connectRDP({ host: '127.0.0.1', port: 3389, username: 'admin' });
  assert.deepEqual(lastCall().args, { config: { host: '127.0.0.1', port: 3389, username: 'admin' } });

  resetMockTauri();
  mockInvoke('connect_ssh', 'ssh-session');
  await connectSSH({ connectionId: 'c1', host: '127.0.0.1', port: 22, username: 'admin', authType: 'password' });
  assert.deepEqual(lastCall().args, {
    config: { host: '127.0.0.1', port: 22, username: 'admin', authType: 'password', credentialId: null },
  });

  resetMockTauri();
  mockInvoke('get_session_state', 'connected');
  await getSessionState('s1');
  assert.deepEqual(lastCall(), { method: 'get_session_state', args: { id: 's1' } });

  resetMockTauri();
  mockInvoke('disconnect_session', undefined);
  await disconnectSession('s1');
  assert.deepEqual(lastCall(), { method: 'disconnect_session', args: { id: 's1' } });

  resetMockTauri();
  mockInvoke('flush_session_output', ['hello']);
  await flushSessionOutput('s1');
  assert.deepEqual(lastCall(), { method: 'flush_session_output', args: { id: 's1' } });

  resetMockTauri();
  mockInvoke('terminal_input', undefined);
  await terminalInput('s1', 'ls\n');
  assert.deepEqual(lastCall(), { method: 'terminal_input', args: { id: 's1', data: 'ls\n' } });

  resetMockTauri();
  mockInvoke('terminal_resize', undefined);
  await terminalResize('s1', 120, 40);
  assert.deepEqual(lastCall(), { method: 'terminal_resize', args: { id: 's1', cols: 120, rows: 40 } });
});

test('ipc: SFTP wrappers use expected payloads', async () => {
  resetMockTauri();
  mockInvoke('list_sftp_dir', [makeSftpFileInfo()]);
  await listSftpDir('s1', '/tmp');
  assert.deepEqual(lastCall(), { method: 'list_sftp_dir', args: { sessionId: 's1', path: '/tmp' } });

  resetMockTauri();
  mockInvoke('sftp_download', [1, 2, 3]);
  await sftpDownload('s1', '/tmp/a.txt');
  assert.deepEqual(lastCall(), { method: 'sftp_download', args: { sessionId: 's1', remotePath: '/tmp/a.txt' } });

  resetMockTauri();
  mockInvoke('sftp_upload', undefined);
  await sftpUpload('s1', '/tmp/a.txt', [1]);
  assert.deepEqual(lastCall(), { method: 'sftp_upload', args: { sessionId: 's1', remotePath: '/tmp/a.txt', data: [1] } });

  resetMockTauri();
  mockInvoke('sftp_mkdir', undefined);
  await sftpMkdir('s1', '/tmp/new');
  assert.deepEqual(lastCall(), { method: 'sftp_mkdir', args: { sessionId: 's1', path: '/tmp/new' } });

  resetMockTauri();
  mockInvoke('sftp_rm', undefined);
  await sftpRm('s1', '/tmp/new');
  assert.deepEqual(lastCall(), { method: 'sftp_rm', args: { sessionId: 's1', path: '/tmp/new' } });

  resetMockTauri();
  mockInvoke('sftp_rename', undefined);
  await sftpRename('s1', '/tmp/a', '/tmp/b');
  assert.deepEqual(lastCall(), { method: 'sftp_rename', args: { sessionId: 's1', oldPath: '/tmp/a', newPath: '/tmp/b' } });

  resetMockTauri();
  mockInvoke('sftp_stat', makeSftpFileInfo());
  await sftpStat('s1', '/tmp/b');
  assert.deepEqual(lastCall(), { method: 'sftp_stat', args: { sessionId: 's1', path: '/tmp/b' } });
});

test('ipc: tunnel wrappers convert nested config to snake_case', async () => {
  resetMockTauri();
  mockInvoke('list_tunnels', []);
  await listTunnels('s1');
  assert.deepEqual(lastCall(), { method: 'list_tunnels', args: { sessionId: 's1' } });

  resetMockTauri();
  const config = makeTunnelConfig();
  mockInvoke('create_tunnel', config);
  await createTunnel(config, 's1');
  assert.equal(lastCall().method, 'create_tunnel');
  assert.deepEqual(lastCall().args?.config, config);

  resetMockTauri();
  mockInvoke('stop_tunnel', undefined);
  await stopTunnel('t1');
  assert.deepEqual(lastCall(), { method: 'stop_tunnel', args: { id: 't1' } });
});

test('ipc: terminal event listeners use session-scoped event names', async () => {
  resetMockTauri();
  const outputs: string[] = [];
  const unlisten = await listenToTerminalOutput('s1', (event) => outputs.push(event.data));
  emitTauriEvent('terminal:output-s1', { id: 's1', data: 'hello' });
  assert.deepEqual(outputs, ['hello']);
  unlisten();
  emitTauriEvent('terminal:output-s1', { id: 's1', data: 'ignored' });
  assert.deepEqual(outputs, ['hello']);

  const seen: string[] = [];
  await listenToTerminalConnected('s1', () => seen.push('connected'));
  await listenToTerminalError('s1', () => seen.push('error'));
  await listenToTerminalExit('s1', () => seen.push('exit'));
  emitTauriEvent('terminal:connected-s1', { id: 's1', cols: 80, rows: 24 });
  emitTauriEvent('terminal:error-s1', { id: 's1', error: 'boom' });
  emitTauriEvent('terminal:exit-s1', { id: 's1' });
  assert.deepEqual(seen, ['connected', 'error', 'exit']);
});

runTests('IPC contract tests')
  .catch(failOnError)
  .finally(restoreMockTauri);
