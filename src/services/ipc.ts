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

// ─── Helper: camelCase → snake_case (used by preparePayload) ──────────────────

function toSnakeCase(str: string): string {
  return str.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
}

function preparePayload(
  obj: Record<string, unknown> | object | unknown[],
): Record<string, unknown> {
  if (Array.isArray(obj)) {
    return obj.map((item) =>
      typeof item === 'object' && item !== null ? preparePayload(item as Record<string, unknown>) : item,
    ) as unknown as Record<string, unknown>;
  }
  const converted: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(obj)) {
    const snakeKey = toSnakeCase(key);
    if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      converted[snakeKey] = preparePayload(value as Record<string, unknown>);
    } else {
      converted[snakeKey] = value;
    }
  }
  return converted;
}

// ─── Connections ────────────────────────────────────────────────────────────

export async function listConnections(): Promise<IpcResult<Connection[]>> {
  return tauriInvoke<Connection[]>('list_connections');
}

import { logJson } from './logging';

export async function createConnection(data: ConnectionFormData): Promise<IpcResult<Connection>> {
  const payload = preparePayload(data);
  // tags: string[] → JSON string for Rust's Option<String>
  if (data.tags && Array.isArray(data.tags)) {
    payload.tags = JSON.stringify(data.tags);
  }
  // Log the actual IPC payload for debugging
  await logJson('createConnection_ipc_payload', payload);
  // startup_commands is already a newline-delimited string in ConnectionFormData
  return tauriInvoke<Connection>('create_connection', { req: payload });
}

export async function updateConnection(data: Connection): Promise<IpcResult<Connection>> {
  const payload = preparePayload(data);
  // tags & startupCommands: string[] → JSON string for Rust's Option<String>
  if (data.tags && Array.isArray(data.tags)) {
    payload.tags = JSON.stringify(data.tags);
  }
  if (data.startupCommands && Array.isArray(data.startupCommands)) {
    payload.startup_commands = JSON.stringify(data.startupCommands);
  }
  return tauriInvoke<Connection>('update_connection', { conn: payload });
}

export async function deleteConnection(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('delete_connection', { id });
}

// ─── Folders ────────────────────────────────────────────────────────────────

export async function listFolders(): Promise<IpcResult<Folder[]>> {
  return tauriInvoke<Folder[]>('list_folders');
}

export async function createFolder(data: Omit<Folder, 'id' | 'createdAt' | 'updatedAt'>): Promise<IpcResult<Folder>> {
  return tauriInvoke<Folder>('create_folder', preparePayload(data));
}

export async function updateFolder(data: Folder): Promise<IpcResult<Folder>> {
  return tauriInvoke<Folder>('update_folder', preparePayload(data));
}

export async function deleteFolder(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('delete_folder', { id });
}

// ─── Credentials ────────────────────────────────────────────────────────────

export async function getCredentials(): Promise<IpcResult<Credential[]>> {
  return tauriInvoke<Credential[]>('get_credentials');
}

export async function saveCredential(data: Omit<Credential, 'id' | 'createdAt' | 'updatedAt'>): Promise<IpcResult<Credential>> {
  return tauriInvoke<Credential>('save_credential', preparePayload(data));
}

export async function deleteCredential(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('delete_credential', { id });
}

// ─── File Picker ────────────────────────────────────────────────────────────────

export async function pickSSHKeyFile(): Promise<IpcResult<string>> {
  return tauriInvoke<string>('pick_ssh_key_file');
}

// ─── Terminal Sessions ──────────────────────────────────────────────────────

export async function connectSSH(config: {
  connectionId: string;
  host: string;
  port: number;
  username: string;
  authType: string;
  credentialId?: string;
}): Promise<IpcResult<string>> {
  // Build payload with explicit snake_case keys — no preparePayload to avoid nesting bugs
  const payload: Record<string, unknown> = {
    connection_id: config.connectionId,
    host: config.host,
    port: Number(config.port),
    username: config.username,
    auth_type: String(config.authType),
    credential_id: config.credentialId ?? null,
  };
  console.log('[connectSSH] calling with config:', JSON.stringify(payload));
  return tauriInvoke<string>('connect_ssh', { config: payload });
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
  return tauriInvoke<SftpFileInfo[]>('list_sftp_dir', { session_id: sessionId, path });
}

export async function sftpDownload(sessionId: string, remotePath: string): Promise<IpcResult<number[]>> {
  return tauriInvoke<number[]>('sftp_download', { session_id: sessionId, remote_path: remotePath });
}

export async function sftpUpload(sessionId: string, remotePath: string, data: number[]): Promise<IpcResult<void>> {
  return tauriInvoke<void>('sftp_upload', { session_id: sessionId, remote_path: remotePath, data });
}

export async function sftpMkdir(sessionId: string, path: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('sftp_mkdir', { session_id: sessionId, path });
}

export async function sftpRm(sessionId: string, path: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('sftp_rm', { session_id: sessionId, path });
}

export async function sftpRename(sessionId: string, oldPath: string, newPath: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('sftp_rename', { session_id: sessionId, old_path: oldPath, new_path: newPath });
}

export async function sftpStat(sessionId: string, path: string): Promise<IpcResult<SftpFileInfo>> {
  return tauriInvoke<SftpFileInfo>('sftp_stat', { session_id: sessionId, path });
}

// ─── SSH Tunnels / Port Forwarding ──────────────────────────────────────────

export async function listTunnels(sessionId?: string): Promise<IpcResult<TunnelConfig[]>> {
  return tauriInvoke<TunnelConfig[]>('list_tunnels', sessionId ? { session_id: sessionId } : undefined);
}

export async function createTunnel(config: TunnelConfig, sessionId: string): Promise<IpcResult<TunnelConfig>> {
  // Flatten TunnelConfig fields + session_id at top level for Rust's create_tunnel(config, session_id)
  const payload = { ...preparePayload(config), session_id: sessionId };
  return tauriInvoke<TunnelConfig>('create_tunnel', payload);
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

export async function listenToTerminalConnected(
  id: string,
  callback: (payload: { id: string; cols: number; rows: number }) => void,
): Promise<UnlistenFn> {
  return listen<{ id: string; cols: number; rows: number }>(`terminal:connected-${id}`, (event) => {
    callback(event.payload);
  });
}

export async function listenToTerminalError(
  id: string,
  callback: (payload: { id: string; error: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ id: string; error: string }>(`terminal:error-${id}`, (event) => {
    callback(event.payload);
  });
}

export async function listenToTerminalExit(
  id: string,
  callback: (payload: { id: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ id: string }>(`terminal:exit-${id}`, (event) => {
    callback(event.payload);
  });
}
