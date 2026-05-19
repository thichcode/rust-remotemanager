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

type InvokeAdapter = <T>(method: string, args?: Record<string, unknown>) => Promise<T>;
type ListenAdapter = <T>(event: string, handler: (event: { payload: T }) => void) => Promise<UnlistenFn>;

let invokeAdapter: InvokeAdapter = invoke;
let listenAdapter: ListenAdapter = listen;

export function __setIpcTestAdapters(adapters: { invoke?: InvokeAdapter; listen?: ListenAdapter }): void {
  if (adapters.invoke) invokeAdapter = adapters.invoke;
  if (adapters.listen) listenAdapter = adapters.listen;
}

export function __resetIpcTestAdapters(): void {
  invokeAdapter = invoke;
  listenAdapter = listen;
}

async function tauriInvoke<T>(method: string, args?: Record<string, unknown>): Promise<IpcResult<T>> {
  try {
    const result = await invokeAdapter<T>(method, args);
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
  const payload = { ...data } as Record<string, unknown>;
  if (data.tags && Array.isArray(data.tags)) {
    payload.tags = JSON.stringify(data.tags);
  }
  await logJson('createConnection_ipc_payload', payload);
  return tauriInvoke<Connection>('create_connection', { req: payload });
}

export async function updateConnection(data: Connection): Promise<IpcResult<Connection>> {
  const payload = { ...data } as Record<string, unknown>;
  if (data.tags && Array.isArray(data.tags)) {
    payload.tags = JSON.stringify(data.tags);
  }
  if (data.startupCommands && Array.isArray(data.startupCommands)) {
    payload.startupCommands = JSON.stringify(data.startupCommands);
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
  return tauriInvoke<Folder>('create_folder', data as unknown as Record<string, unknown>);
}

export async function updateFolder(data: Folder): Promise<IpcResult<Folder>> {
  return tauriInvoke<Folder>('update_folder', data as unknown as Record<string, unknown>);
}

export async function deleteFolder(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('delete_folder', { id });
}

// ─── Credentials ────────────────────────────────────────────────────────────

export async function getCredentials(): Promise<IpcResult<Credential[]>> {
  return tauriInvoke<Credential[]>('list_credentials');
}

// Tauri v2 flat params expect camelCase keys — send manually, NOT via preparePayload
export async function saveCredential(data: {
  name: string;
  authType: string;
  username?: string;
  password?: string;
  privateKey?: string;
  keyPath?: string;
  passphraseProtected?: boolean;
}): Promise<IpcResult<Credential>> {
  return tauriInvoke<Credential>('save_credential', {
    name: data.name,
    authType: data.authType,
    username: data.username,
    password: data.password,
    privateKey: data.privateKey,
    keyPath: data.keyPath,
    passphraseProtected: data.passphraseProtected,
  });
}

export async function deleteCredential(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('delete_credential', { id });
}

// ─── File Picker ────────────────────────────────────────────────────────────────

export async function pickSSHKeyFile(): Promise<IpcResult<string>> {
  return tauriInvoke<string>('pick_ssh_key_file');
}

// ─── RDP Connections ────────────────────────────────────────────────────────

export async function connectRDP(config: {
  host: string;
  port: number;
  username?: string;
}): Promise<IpcResult<string>> {
  const payload = {
    host: config.host,
    port: config.port,
    username: config.username ?? null,
  };
  return tauriInvoke<string>('connect_rdp', { config: payload });
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
  const payload: Record<string, unknown> = {
    host: config.host,
    port: Number(config.port),
    username: config.username,
    authType: String(config.authType),
    credentialId: config.credentialId ?? null,
  };
  return tauriInvoke<string>('connect_ssh', { config: payload });
}

export async function getSessionState(id: string): Promise<IpcResult<string | null>> {
  return tauriInvoke<string | null>('get_session_state', { id });
}

export async function disconnectSession(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('disconnect_session', { id });
}

export async function flushSessionOutput(id: string): Promise<IpcResult<string[]>> {
  return tauriInvoke<string[]>('flush_session_output', { id });
}

export async function terminalInput(id: string, data: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('terminal_input', { id, data });
}

export async function terminalResize(id: string, cols: number, rows: number): Promise<IpcResult<void>> {
  return tauriInvoke<void>('terminal_resize', { id, cols, rows });
}

// ─── Vault ──────────────────────────────────────────────────────────────────

export async function vaultStatus(): Promise<IpcResult<boolean>> {
  return tauriInvoke<boolean>('vault_status');
}

export async function vaultUnlock(password: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('vault_unlock', { password });
}

export async function vaultLock(): Promise<IpcResult<void>> {
  return tauriInvoke<void>('vault_lock');
}

// ─── Settings ───────────────────────────────────────────────────────────────

export async function getSettings(): Promise<IpcResult<Record<string, string>>> {
  return tauriInvoke<Record<string, string>>('get_settings');
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
  return tauriInvoke<TunnelConfig[]>('list_tunnels', sessionId ? { sessionId } : undefined);
}

export async function createTunnel(config: TunnelConfig, sessionId: string): Promise<IpcResult<TunnelConfig>> {
  // Tauri v2 flat params: sessionId (camelCase for Rust session_id) + config (struct)
  return tauriInvoke<TunnelConfig>('create_tunnel', {
    sessionId,
    config: preparePayload(config),
  });
}

export async function stopTunnel(id: string): Promise<IpcResult<void>> {
  return tauriInvoke<void>('stop_tunnel', { id });
}

// ─── Event Listeners ────────────────────────────────────────────────────────

export async function listenToTerminalOutput(
  id: string,
  callback: (event: TerminalOutputEvent) => void,
): Promise<UnlistenFn> {
  return listenAdapter<TerminalOutputEvent>(`terminal:output-${id}`, (event) => {
    callback(event.payload);
  });
}

export async function listenToTerminalConnected(
  id: string,
  callback: (payload: { id: string; cols: number; rows: number }) => void,
): Promise<UnlistenFn> {
  return listenAdapter<{ id: string; cols: number; rows: number }>(`terminal:connected-${id}`, (event) => {
    callback(event.payload);
  });
}

export async function listenToTerminalError(
  id: string,
  callback: (payload: { id: string; error: string }) => void,
): Promise<UnlistenFn> {
  return listenAdapter<{ id: string; error: string }>(`terminal:error-${id}`, (event) => {
    callback(event.payload);
  });
}

export async function listenToTerminalExit(
  id: string,
  callback: (payload: { id: string }) => void,
): Promise<UnlistenFn> {
  return listenAdapter<{ id: string }>(`terminal:exit-${id}`, (event) => {
    callback(event.payload);
  });
}
