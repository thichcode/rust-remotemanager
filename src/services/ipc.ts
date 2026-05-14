import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
  Connection,
  Folder,
  Credential,
  Setting,
  TerminalSession,
  IpcResult,
  TerminalOutputEvent,
  ConnectionFormData,
  SftpFileInfo,
  TunnelConfig,
} from './types';

// ─── Helper ─────────────────────────────────────────────────────────────────

async function tauriInvoke<T>(method: string, args?: Record<string, unknown>): Promise<IpcResult<T>> {
  try {
    const result = await invoke<T>(method, args);
    return { success: true, data: result };
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : String(error);
    console.error(`[IPC] ${method} failed:`, message);
    return { success: false, error: message };
  }
}

// ─── Connections ────────────────────────────────────────────────────────────

export async function listConnections(): Promise<IpcResult<Connection[]>> {
  return tauriInvoke<Connection[]>('list_connections');
}

export async function createConnection(data: ConnectionFormData): Promise<IpcResult<Connection>> {
  return tauriInvoke<Connection>('create_connection', { data });
}

export async function updateConnection(data: Connection): Promise<IpcResult<Connection>> {
  return tauriInvoke<Connection>('update_connection', { data });
}

export async function deleteConnection(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('delete_connection', { id });
}

// ─── Folders ────────────────────────────────────────────────────────────────

export async function listFolders(): Promise<IpcResult<Folder[]>> {
  return tauriInvoke<Folder[]>('list_folders');
}

export async function createFolder(data: Omit<Folder, 'id' | 'createdAt' | 'updatedAt'>): Promise<IpcResult<Folder>> {
  return tauriInvoke<Folder>('create_folder', { data });
}

export async function updateFolder(data: Folder): Promise<IpcResult<Folder>> {
  return tauriInvoke<Folder>('update_folder', { data });
}

export async function deleteFolder(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('delete_folder', { id });
}

// ─── Credentials ────────────────────────────────────────────────────────────

export async function getCredentials(): Promise<IpcResult<Credential[]>> {
  return tauriInvoke<Credential[]>('get_credentials');
}

export async function saveCredential(data: Omit<Credential, 'id' | 'createdAt' | 'updatedAt'>): Promise<IpcResult<Credential>> {
  return tauriInvoke<Credential>('save_credential', { data });
}

export async function deleteCredential(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('delete_credential', { id });
}

// ─── Terminal Sessions ──────────────────────────────────────────────────────

export async function connectSSH(config: {
  connectionId: string;
  host: string;
  port: number;
  username: string;
  authType: string;
  credentialId?: string;
}): Promise<IpcResult<TerminalSession>> {
  return tauriInvoke<TerminalSession>('connect_ssh', { config });
}

export async function disconnectSession(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('disconnect_session', { id });
}

export async function terminalInput(id: string, data: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('terminal_input', { id, data });
}

export async function terminalResize(id: string, cols: number, rows: number): Promise<IpcResult<void>> {
  return tauriInvoke<void>('terminal_resize', { id, cols, rows });
}

// ─── Vault ──────────────────────────────────────────────────────────────────

export async function vaultStatus(): Promise<IpcResult<{ locked: boolean }>> {
  return tauriInvoke<{ locked: boolean }>('vault_status');
}

export async function vaultUnlock(password: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('vault_unlock', { password });
}

export async function vaultLock(): Promise<IpcResult<void>> {
  return tauriInvoke<void>('vault_lock');
}

// ─── Settings ───────────────────────────────────────────────────────────────

export async function getSettings(): Promise<IpcResult<Setting[]>> {
  return tauriInvoke<Setting[]>('get_settings');
}

export async function updateSetting(key: string, value: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('update_setting', { key, value });
}

// ─── SFTP File Browser ───────────────────────────────────────────────────────

export async function listSftpDir(sessionId: string, path: string): Promise<IpcResult<SftpFileInfo[]>> {
  return tauriInvoke<SftpFileInfo[]>('list_sftp_dir', { sessionId, path });
}

export async function sftpDownload(sessionId: string, remotePath: string): Promise<IpcResult<number[]>> {
  return tauriInvoke<number[]>('sftp_download', { sessionId, remotePath });
}

export async function sftpUpload(sessionId: string, remotePath: string, data: number[]): Promise<IpcResult<void>> {
  return tauriInvoke<void>('sftp_upload', { sessionId, remotePath, data });
}

export async function sftpMkdir(sessionId: string, path: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('sftp_mkdir', { sessionId, path });
}

export async function sftpRm(sessionId: string, path: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('sftp_rm', { sessionId, path });
}

export async function sftpRename(sessionId: string, oldPath: string, newPath: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('sftp_rename', { sessionId, oldPath, newPath });
}

export async function sftpStat(sessionId: string, path: string): Promise<IpcResult<SftpFileInfo>> {
  return tauriInvoke<SftpFileInfo>('sftp_stat', { sessionId, path });
}

// ─── SSH Tunnels / Port Forwarding ──────────────────────────────────────────

export async function listTunnels(sessionId?: string): Promise<IpcResult<TunnelConfig[]>> {
  return tauriInvoke<TunnelConfig[]>('list_tunnels', { sessionId });
}

export async function createTunnel(config: TunnelConfig, sessionId: string): Promise<IpcResult<TunnelConfig>> {
  return tauriInvoke<TunnelConfig>('create_tunnel', { config, sessionId });
}

export async function stopTunnel(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('stop_tunnel', { id });
}

// ─── Event Listeners ────────────────────────────────────────────────────────

export async function listenToTerminalOutput(
  id: string,
  callback: (event: TerminalOutputEvent) => void,
): Promise<UnlistenFn> {
  return listen<TerminalOutputEvent>(`terminal:output-${id}`, (event) => {
    callback(event.payload);
  });
}
