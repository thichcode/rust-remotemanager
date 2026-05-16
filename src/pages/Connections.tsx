import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Plus,
  FolderPlus,
  Download,
  Upload,
  RefreshCw,
  Search,
  FolderOpen,
  Terminal,
} from 'lucide-react';
import { useConnectionStore } from '../stores/connectionStore';
import { useUiStore } from '../stores/uiStore';
import { useConnections } from '../hooks/useConnections';
import ConnectionList from '../components/connections/ConnectionList';
import FolderTree from '../components/connections/FolderTree';
import ConnectionForm from '../components/connections/ConnectionForm';
import type { Connection, ConnectionFormData } from '../services/types';
import { connectSSH } from '../services/ipc';
import toast from 'react-hot-toast';
import { useSessionStore } from '../stores/sessionStore';

export default function Connections() {
  const {
    connections,
    folders,
    loading,
    refreshing,
    refresh,
    createConnection,
    updateConnection,
    deleteConnection,
    createFolder,
    updateFolder,
    deleteFolder,
  } = useConnections();

  const {
    searchTerm,
    setSearchTerm,
    filterType,
    setFilterType,
    selectedConnectionId,
    setSelectedConnectionId,
  } = useConnectionStore();

  const {
    showConnectionForm,
    editingConnection,
    openConnectionForm,
    closeConnectionForm,
    showFolderDialog,
    editingFolder,
    openFolderDialog,
    closeFolderDialog,
  } = useUiStore();

  const { addSession } = useSessionStore();
  const navigate = useNavigate();

  const [selectedFolderId, setSelectedFolderId] = useState<string | null>(null);
  const [folderNameInput, setFolderNameInput] = useState('');

  // Filter connections by selected folder
  const filteredConnections = selectedFolderId
    ? connections.filter((c) => c.folderId === selectedFolderId)
    : connections;

  const handleCreateConnection = () => {
    openConnectionForm();
  };

  const handleEditConnection = (connection: Connection) => {
    openConnectionForm(connection);
  };

  const handleDeleteConnection = async (connection: Connection) => {
    if (window.confirm(`Delete connection "${connection.name}"?`)) {
      await deleteConnection(connection.id);
    }
  };

  const handleToggleFavorite = async (connection: Connection) => {
    await updateConnection({
      ...connection,
      isFavorite: !connection.isFavorite,
    });
  };

  const handleSaveConnection = async (data: ConnectionFormData) => {
    if (editingConnection) {
      const payload: Connection = {
        ...editingConnection,
        ...data,
        // credentialId must come from form data, not stale editingConnection
        credentialId: data.credentialId,
        startupCommands: data.startupCommands
          ? data.startupCommands.split('\n').filter(Boolean)
          : [],
      };
      await updateConnection(payload);
    } else {
      await createConnection(data);
    }
    closeConnectionForm();
  };

  const handleNewFolder = (parentId?: string) => {
    openFolderDialog();
    setFolderNameInput('');
  };

  const handleEditFolder = (folder: { id: string; name: string }) => {
    openFolderDialog({
      id: folder.id,
      name: folder.name,
      parentId: undefined,
      sortOrder: 0,
      createdAt: '',
      updatedAt: '',
    });
    setFolderNameInput(folder.name);
  };

  const handleDeleteFolder = async (folder: { id: string; name: string }) => {
    if (window.confirm(`Delete folder "${folder.name}" and all its contents?`)) {
      await deleteFolder(folder.id);
    }
  };

  const handleSaveFolder = async () => {
    if (!folderNameInput.trim()) return;

    if (editingFolder) {
      await updateFolder({
        ...editingFolder,
        name: folderNameInput.trim(),
      });
    } else {
      await createFolder({
        name: folderNameInput.trim(),
        sortOrder: folders.length,
      });
    }
    closeFolderDialog();
  };

  const handleConnect = async (connection: Connection) => {
    console.log('[Connections] handleConnect start:', connection.name, connection.host);
    try {
      const result = await connectSSH({
        connectionId: connection.id,
        host: connection.host,
        port: connection.port,
        username: connection.username,
        authType: connection.authType,
        credentialId: connection.credentialId,
      });

      console.log('[Connections] connectSSH result:', JSON.stringify(result));

      if (result.success && result.data) {
        console.log('[Connections] connectSSH success, sessionId:', result.data);
        // Rust returns session_id (String), not TerminalSession object
        addSession({
          id: result.data,
          connectionId: connection.id,
          state: 'connecting',
          createdAt: new Date().toISOString(),
        });
        console.log('[Connections] navigating to /terminal/' + result.data);
        navigate(`/terminal/${result.data}`);
      } else {
        console.error('[Connections] connectSSH failed:', result.error);
        toast.error(result.error ?? 'Failed to connect');
      }
    } catch (err: any) {
      console.error('[Connections] connectSSH exception:', err?.message ?? String(err));
      toast.error(err?.message ?? 'Connection failed');
    }
  };

  return (
    <div className="h-full flex">
      {/* Left panel: Folder tree */}
      <div className="w-56 border-r border-[var(--border)] bg-[var(--bg-secondary)] flex flex-col flex-shrink-0">
        <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--border)]">
          <span className="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)]">
            Folders
          </span>
          <button
            onClick={() => handleNewFolder()}
            className="p-1 rounded-md text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
            title="New folder"
          >
            <FolderPlus size={14} />
          </button>
        </div>
        <div className="flex-1 overflow-y-auto p-2">
          <FolderTree
            folders={folders}
            connections={connections}
            selectedFolderId={selectedFolderId}
            onSelectFolder={(id) => setSelectedFolderId(id)}
            onNewFolder={handleNewFolder}
            onEditFolder={handleEditFolder}
            onDeleteFolder={handleDeleteFolder}
          />
        </div>
      </div>

      {/* Right panel: Connection list */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Toolbar */}
        <div className="flex items-center justify-between px-6 py-3 border-b border-[var(--border)] bg-[var(--bg-secondary)]">
          <div className="flex items-center gap-2">
            <button
              onClick={handleCreateConnection}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--accent)] text-white text-sm font-medium hover:bg-[var(--accent-hover)] active:scale-[0.98] transition-all"
            >
              <Plus size={16} />
              New Connection
            </button>
            <button
              onClick={() => handleNewFolder()}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-lg border border-[var(--border)] text-sm font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
            >
              <FolderPlus size={16} />
              New Folder
            </button>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={refresh}
              disabled={refreshing}
              className="p-2 rounded-lg border border-[var(--border)] text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors disabled:opacity-50"
              title="Refresh"
            >
              <RefreshCw size={16} className={refreshing ? 'animate-spin' : ''} />
            </button>
            <button
              className="p-2 rounded-lg border border-[var(--border)] text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
              title="Import"
            >
              <Download size={16} />
            </button>
            <button
              className="p-2 rounded-lg border border-[var(--border)] text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
              title="Export"
            >
              <Upload size={16} />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
          {loading ? (
            <div className="flex items-center justify-center h-full">
              <div className="flex items-center gap-3 text-[var(--text-muted)]">
                <RefreshCw size={20} className="animate-spin" />
                <span className="text-sm">Loading connections...</span>
              </div>
            </div>
          ) : (
            <ConnectionList
              connections={filteredConnections}
              searchTerm={searchTerm}
              onSearchChange={setSearchTerm}
              filterType={filterType}
              onFilterTypeChange={setFilterType}
              onConnect={handleConnect}
              onEdit={handleEditConnection}
              onDelete={handleDeleteConnection}
              onToggleFavorite={handleToggleFavorite}
            />
          )}
        </div>
      </div>

      {/* Connection Form Modal */}
      {showConnectionForm && (
        <ConnectionForm
          editingConnection={editingConnection}
          onSave={handleSaveConnection}
          onCancel={closeConnectionForm}
        />
      )}

      {/* Folder Dialog */}
      {showFolderDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
          <div className="w-full max-w-sm rounded-2xl border border-[var(--border)] bg-[var(--bg-secondary)] shadow-2xl">
            <div className="px-5 py-4 border-b border-[var(--border)]">
              <h3 className="text-base font-bold text-[var(--text-primary)]">
                {editingFolder ? 'Rename Folder' : 'New Folder'}
              </h3>
            </div>
            <div className="px-5 py-4">
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                Folder Name
              </label>
              <input
                type="text"
                value={folderNameInput}
                onChange={(e) => setFolderNameInput(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleSaveFolder();
                  if (e.key === 'Escape') closeFolderDialog();
                }}
                placeholder="e.g., Production Servers"
                autoFocus
                className="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors"
              />
            </div>
            <div className="flex items-center justify-end gap-3 px-5 py-3 border-t border-[var(--border)]">
              <button
                onClick={closeFolderDialog}
                className="px-4 py-2 rounded-lg border border-[var(--border)] text-sm font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveFolder}
                disabled={!folderNameInput.trim()}
                className="px-4 py-2 rounded-lg bg-[var(--accent)] text-white text-sm font-medium hover:bg-[var(--accent-hover)] disabled:opacity-50 transition-all"
              >
                {editingFolder ? 'Rename' : 'Create'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
