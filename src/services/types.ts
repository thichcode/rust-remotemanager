// ─── Enums ──────────────────────────────────────────────────────────────────

export enum ConnectionType {
  SSH = 'ssh',
  RDP = 'rdp',
  Serial = 'serial',
}

export enum AuthType {
  Password = 'password',
  Key = 'key',
  Agent = 'agent',
}

export enum ProxyType {
  None = 'none',
  Socks5 = 'socks5',
  Http = 'http',
}

export enum TunnelType {
  Local = 'local',
  Remote = 'remote',
  Dynamic = 'dynamic',
}

// ─── Core Data Models ───────────────────────────────────────────────────────

export interface Connection {
  id: string;
  name: string;
  type: ConnectionType;
  folderId?: string;
  host: string;
  port: number;
  username: string;
  credentialId?: string;
  authType: AuthType;
  proxyType?: ProxyType;
  proxyHost?: string;
  proxyPort?: number;
  proxyUsername?: string;
  tags?: string[];
  notes?: string;
  startupCommands?: string[];
  keepaliveInterval?: number;
  isFavorite: boolean;
  color?: string;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface Folder {
  id: string;
  name: string;
  parentId?: string;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
}

export interface Credential {
  id: string;
  name: string;
  authType: AuthType;
  username?: string;
  keyPath?: string;
  createdAt: string;
  updatedAt: string;
}

export type TerminalSessionState = 'connecting' | 'connected' | 'disconnected' | 'error';

export interface TerminalSession {
  id: string;
  connectionId: string;
  state: TerminalSessionState;
  createdAt: string;
}

export interface Setting {
  key: string;
  value: string;
}

export interface SessionLog {
  id: string;
  connectionId: string;
  startedAt: string;
  endedAt?: string;
  bytesSent: number;
  bytesReceived: number;
}

export interface PortForward {
  id: string;
  localPort: number;
  remoteHost: string;
  remotePort: number;
  type: TunnelType;
  active: boolean;
}

// ─── IPC Result Types ───────────────────────────────────────────────────────

export interface IpcResult<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
}

// ─── Event Payloads ─────────────────────────────────────────────────────────

export interface TerminalOutputEvent {
  id: string;
  data: string;
}

export interface TerminalResizePayload {
  id: string;
  cols: number;
  rows: number;
}

// ─── Form Types ─────────────────────────────────────────────────────────────

export interface ConnectionFormData {
  name: string;
  type: ConnectionType;
  folderId?: string;
  host: string;
  port: number;
  username: string;
  credentialId?: string;
  authType: AuthType;
  proxyType?: ProxyType;
  proxyHost?: string;
  proxyPort?: number;
  proxyUsername?: string;
  tags: string[];
  notes: string;
  startupCommands: string;
  keepaliveInterval: number;
  isFavorite: boolean;
  color?: string;
}

// ─── Navigation ─────────────────────────────────────────────────────────────

export interface NavItem {
  label: string;
  path: string;
  icon: string;
  badge?: number;
}
