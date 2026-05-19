import {
  AuthType,
  ConnectionType,
  TunnelType,
  type Connection,
  type ConnectionFormData,
  type Credential,
  type Folder,
  type SftpFileInfo,
  type TerminalSession,
  type TunnelConfig,
} from '../../src/services/types';

function now(): string {
  return new Date().toISOString();
}

export function makeConnection(id = 'conn-1', overrides: Partial<Connection> = {}): Connection {
  return {
    id,
    name: `Server ${id}`,
    type: ConnectionType.SSH,
    host: '127.0.0.1',
    port: 22,
    username: 'admin',
    authType: AuthType.Password,
    sortOrder: 0,
    createdAt: now(),
    updatedAt: now(),
    ...overrides,
  };
}

export function makeConnectionFormData(overrides: Partial<ConnectionFormData> = {}): ConnectionFormData {
  return {
    name: 'Test Server',
    type: ConnectionType.SSH,
    host: '127.0.0.1',
    port: 22,
    username: 'admin',
    authType: AuthType.Password,
    tags: [],
    keyPath: '',
    notes: '',
    startupCommands: '',
    keepaliveInterval: 0,
    isFavorite: false,
    ...overrides,
  };
}

export function makeFolder(id = 'folder-1', overrides: Partial<Folder> = {}): Folder {
  return {
    id,
    name: `Folder ${id}`,
    sortOrder: 0,
    createdAt: now(),
    updatedAt: now(),
    ...overrides,
  };
}

export function makeCredential(id = 'cred-1', overrides: Partial<Credential> = {}): Credential {
  return {
    id,
    name: `Credential ${id}`,
    authType: AuthType.Password,
    username: 'admin',
    createdAt: now(),
    updatedAt: now(),
    ...overrides,
  };
}

export function makeTerminalSession(id = 'session-1', overrides: Partial<TerminalSession> = {}): TerminalSession {
  return {
    id,
    connectionId: `conn-${id}`,
    state: 'connecting',
    createdAt: now(),
    ...overrides,
  };
}

export function makeSftpFileInfo(overrides: Partial<SftpFileInfo> = {}): SftpFileInfo {
  return {
    name: 'file.txt',
    path: '/home/hermes/file.txt',
    size: 12,
    is_dir: false,
    is_symlink: false,
    permissions: '644',
    modified: now(),
    ...overrides,
  };
}

export function makeTunnelConfig(overrides: Partial<TunnelConfig> = {}): TunnelConfig {
  return {
    id: 'tunnel-1',
    session_id: 'session-1',
    tunnel_type: TunnelType.Local,
    name: 'Local tunnel',
    local_host: '127.0.0.1',
    local_port: 15432,
    remote_host: '127.0.0.1',
    remote_port: 5432,
    active: true,
    ...overrides,
  };
}
