import { useState, useEffect, useCallback, useRef } from 'react';
import {
  File,
  Folder,
  Upload,
  Download,
  Trash2,
  Edit3,
  ChevronLeft,
  Grid3X3,
  List,
  Plus,
  RefreshCw,
  Loader2,
  Home,
  MoreVertical,
  FolderPlus,
} from 'lucide-react';
import { listSftpDir, sftpDownload, sftpUpload, sftpMkdir, sftpRm, sftpRename } from '../../services/ipc';
import type { SftpFileInfo, IpcResult } from '../../services/types';

interface SftpBrowserProps {
  sessionId: string;
  onClose?: () => void;
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '-';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

function formatPermissions(mode: string): string {
  if (!mode || mode === '0') return '---';
  const num = parseInt(mode, 8);
  const r = (n: number) => ((num >> n) & 1) !== 0;
  const w = (n: number) => ((num >> (n - 1)) & 1) !== 0;
  const x = (n: number) => ((num >> (n - 2)) & 1) !== 0;
  const triple = (shift: number) =>
    `${r(shift + 2) ? 'r' : '-'}${w(shift + 1) ? 'w' : '-'}${x(shift) ? 'x' : '-'}`;
  return `${triple(6)}${triple(3)}${triple(0)}`;
}

function getParentPath(path: string): string {
  if (path === '/' || path === '') return '/';
  const normalized = path.endsWith('/') ? path.slice(0, -1) : path;
  const lastSlash = normalized.lastIndexOf('/');
  return lastSlash <= 0 ? '/' : normalized.slice(0, lastSlash);
}

function joinPath(base: string, name: string): string {
  if (base === '/') return `/${name}`;
  return `${base}/${name}`;
}

interface RenameDialogProps {
  currentName: string;
  onConfirm: (newName: string) => void;
  onCancel: () => void;
}

function RenameDialog({ currentName, onConfirm, onCancel }: RenameDialogProps) {
  const [name, setName] = useState(currentName);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    inputRef.current?.select();
  }, []);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="bg-[var(--bg-secondary)] border border-[var(--border)] rounded-xl shadow-2xl p-5 w-80">
        <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-3">Rename</h3>
        <input
          ref={inputRef}
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') onConfirm(name);
            if (e.key === 'Escape') onCancel();
          }}
          className="w-full bg-[var(--bg-tertiary)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] outline-none focus:border-[var(--accent)] mb-4"
        />
        <div className="flex justify-end gap-2">
          <button
            onClick={onCancel}
            className="px-3 py-1.5 text-xs font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={() => onConfirm(name)}
            disabled={!name.trim() || name.trim() === currentName}
            className="px-3 py-1.5 text-xs font-medium text-white bg-[var(--accent)] hover:bg-[var(--accent-hover)] rounded-lg transition-colors disabled:opacity-40"
          >
            Rename
          </button>
        </div>
      </div>
    </div>
  );
}

interface ContextMenuProps {
  x: number;
  y: number;
  file: SftpFileInfo;
  onRename: (file: SftpFileInfo) => void;
  onDelete: (file: SftpFileInfo) => void;
  onDownload: (file: SftpFileInfo) => void;
  onClose: () => void;
}

function ContextMenu({ x, y, file, onRename, onDelete, onDownload, onClose }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [onClose]);

  return (
    <div
      ref={menuRef}
      className="fixed z-50 bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg shadow-2xl py-1 min-w-[150px]"
      style={{ left: x, top: y }}
    >
      {!file.is_dir && (
        <button
          onClick={() => { onDownload(file); onClose(); }}
          className="w-full flex items-center gap-2 px-3 py-2 text-xs text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
        >
          <Download size={14} className="text-[var(--text-muted)]" />
          Download
        </button>
      )}
      <button
        onClick={() => { onRename(file); onClose(); }}
        className="w-full flex items-center gap-2 px-3 py-2 text-xs text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
      >
        <Edit3 size={14} className="text-[var(--text-muted)]" />
        Rename
      </button>
      <div className="border-t border-[var(--border)] my-1" />
      <button
        onClick={() => { onDelete(file); onClose(); }}
        className="w-full flex items-center gap-2 px-3 py-2 text-xs text-[var(--status-error)] hover:bg-[var(--bg-tertiary)] transition-colors"
      >
        <Trash2 size={14} />
        Delete
      </button>
    </div>
  );
}

export default function SftpBrowser({ sessionId }: SftpBrowserProps) {
  const [currentPath, setCurrentPath] = useState('/');
  const [files, setFiles] = useState<SftpFileInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'list' | 'grid'>('list');
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    file: SftpFileInfo;
  } | null>(null);
  const [renameTarget, setRenameTarget] = useState<SftpFileInfo | null>(null);
  const [creatingDir, setCreatingDir] = useState(false);
  const [newDirName, setNewDirName] = useState('');
  const [uploading, setUploading] = useState(false);

  const loadDirectory = useCallback(async (path: string) => {
    setLoading(true);
    setError(null);
    const result = await listSftpDir(sessionId, path);
    if (result.success && result.data) {
      setFiles(result.data);
    } else {
      setError(result.error || 'Failed to list directory');
    }
    setLoading(false);
  }, [sessionId]);

  useEffect(() => {
    loadDirectory(currentPath);
  }, [currentPath, loadDirectory]);

  const navigateTo = (path: string) => {
    setCurrentPath(path);
    setContextMenu(null);
  };

  const handleFileClick = (file: SftpFileInfo) => {
    if (file.is_dir) {
      navigateTo(file.path);
    }
  };

  const handleContextMenu = (e: React.MouseEvent, file: SftpFileInfo) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY, file });
  };

  const handleDownload = async (file: SftpFileInfo) => {
    if (file.is_dir) return;
    const result = await sftpDownload(sessionId, file.path);
    if (result.success && result.data) {
      const blob = new Blob([new Uint8Array(result.data)]);
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = file.name;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } else {
      setError(result.error || 'Download failed');
    }
  };

  const handleUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setUploading(true);
    setError(null);

    try {
      const buffer = await file.arrayBuffer();
      const data = Array.from(new Uint8Array(buffer));
      const remotePath = joinPath(currentPath, file.name);
      const result = await sftpUpload(sessionId, remotePath, data);
      if (result.success) {
        loadDirectory(currentPath);
      } else {
        setError(result.error || 'Upload failed');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Upload failed');
    }

    setUploading(false);
    e.target.value = '';
  };

  const handleDelete = async (file: SftpFileInfo) => {
    const confirmMsg = file.is_dir
      ? `Delete directory "${file.name}" and all its contents?`
      : `Delete file "${file.name}"?`;
    if (!window.confirm(confirmMsg)) return;

    const result = await sftpRm(sessionId, file.path);
    if (result.success) {
      loadDirectory(currentPath);
    } else {
      setError(result.error || 'Delete failed');
    }
  };

  const handleRename = async (newName: string) => {
    if (!renameTarget) return;
    const newPath = joinPath(getParentPath(renameTarget.path), newName);
    const result = await sftpRename(sessionId, renameTarget.path, newPath);
    if (result.success) {
      setRenameTarget(null);
      loadDirectory(currentPath);
    } else {
      setError(result.error || 'Rename failed');
    }
  };

  const handleCreateDir = async () => {
    if (!newDirName.trim()) return;
    const newPath = joinPath(currentPath, newDirName.trim());
    const result = await sftpMkdir(sessionId, newPath);
    if (result.success) {
      setNewDirName('');
      setCreatingDir(false);
      loadDirectory(currentPath);
    } else {
      setError(result.error || 'Failed to create directory');
    }
  };

  // Breadcrumb segments
  const pathSegments = currentPath === '/'
    ? []
    : currentPath.split('/').filter(Boolean);

  return (
    <div className="flex flex-col h-full bg-[var(--bg-primary)] border-l border-[var(--border)] w-[360px] min-w-[300px]">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[var(--border)] bg-[var(--bg-secondary)]">
        <div className="flex items-center gap-1.5">
          <span className="text-xs font-semibold text-[var(--text-primary)]">SFTP Browser</span>
          <button
            onClick={() => navigateTo(currentPath)}
            className="p-1 rounded text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
            title="Refresh"
          >
            <RefreshCw size={12} className={loading ? 'animate-spin' : ''} />
          </button>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={() => setViewMode(viewMode === 'list' ? 'grid' : 'list')}
            className="p-1 rounded text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
            title={viewMode === 'list' ? 'Grid view' : 'List view'}
          >
            {viewMode === 'list' ? <Grid3X3 size={14} /> : <List size={14} />}
          </button>
        </div>
      </div>

      {/* Breadcrumb nav */}
      <div className="flex items-center gap-1 px-3 py-1.5 border-b border-[var(--border)] bg-[var(--bg-secondary)] overflow-x-auto">
        <button
          onClick={() => navigateTo(getParentPath(currentPath))}
          disabled={currentPath === '/'}
          className="p-1 rounded text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title="Go back"
        >
          <ChevronLeft size={14} />
        </button>
        <button
          onClick={() => navigateTo('/')}
          className="p-1 rounded text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors flex-shrink-0"
          title="Root"
        >
          <Home size={14} />
        </button>
        <div className="flex items-center gap-0.5 text-xs text-[var(--text-muted)] ml-1 overflow-hidden">
          <span className="text-[var(--text-muted)]">/</span>
          {pathSegments.map((segment, i) => {
            const path = '/' + pathSegments.slice(0, i + 1).join('/');
            return (
              <span key={path} className="flex items-center gap-0.5">
                <button
                  onClick={() => navigateTo(path)}
                  className="px-1 py-0.5 rounded text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors truncate max-w-[80px]"
                >
                  {segment}
                </button>
                <span className="text-[var(--text-muted)]">/</span>
              </span>
            );
          })}
        </div>
      </div>

      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-[var(--border)] bg-[var(--bg-secondary)]">
        <div className="flex items-center gap-1">
          <button
            onClick={() => setCreatingDir(true)}
            className="flex items-center gap-1 px-2 py-1 rounded text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
            title="New folder"
          >
            <FolderPlus size={13} />
            <span>New</span>
          </button>
        </div>
        <div className="flex items-center gap-1">
          <label className="flex items-center gap-1 px-2 py-1 rounded text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors cursor-pointer">
            <Upload size={13} />
            <span>{uploading ? 'Uploading...' : 'Upload'}</span>
            <input
              type="file"
              onChange={handleUpload}
              className="hidden"
              disabled={uploading}
            />
          </label>
        </div>
      </div>

      {/* Create directory inline */}
      {creatingDir && (
        <div className="flex items-center gap-2 px-3 py-2 border-b border-[var(--border)] bg-[var(--bg-tertiary)]">
          <input
            type="text"
            value={newDirName}
            onChange={(e) => setNewDirName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleCreateDir();
              if (e.key === 'Escape') { setCreatingDir(false); setNewDirName(''); }
            }}
            placeholder="Folder name..."
            autoFocus
            className="flex-1 bg-[var(--bg-primary)] border border-[var(--border)] rounded px-2 py-1 text-xs text-[var(--text-primary)] placeholder-[var(--text-muted)] outline-none focus:border-[var(--accent)]"
          />
          <button
            onClick={handleCreateDir}
            disabled={!newDirName.trim()}
            className="p-1 rounded text-[var(--accent)] hover:bg-[var(--bg-primary)] transition-colors disabled:opacity-40"
          >
            <Plus size={14} />
          </button>
          <button
            onClick={() => { setCreatingDir(false); setNewDirName(''); }}
            className="p-1 rounded text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-primary)] transition-colors"
          >
            <span className="text-xs">Esc</span>
          </button>
        </div>
      )}

      {/* Error banner */}
      {error && (
        <div className="px-3 py-1.5 bg-[var(--status-error)]/10 border-b border-[var(--status-error)]/20 text-xs text-[var(--status-error)]">
          {error}
          <button
            onClick={() => setError(null)}
            className="ml-2 underline hover:no-underline"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* File listing */}
      <div className="flex-1 overflow-y-auto">
        {loading ? (
          <div className="flex items-center justify-center h-full">
            <Loader2 size={20} className="animate-spin text-[var(--text-muted)]" />
          </div>
        ) : files.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-center px-6">
            <Folder size={32} className="text-[var(--text-muted)] mb-2 opacity-40" />
            <p className="text-xs text-[var(--text-muted)]">Empty directory</p>
          </div>
        ) : viewMode === 'list' ? (
          /* List view */
          <div className="text-xs">
            {/* Header row */}
            <div className="flex items-center px-3 py-1.5 text-[var(--text-muted)] border-b border-[var(--border)] bg-[var(--bg-secondary)] sticky top-0">
              <span className="flex-1">Name</span>
              <span className="w-[70px] text-right">Size</span>
              <span className="w-[80px] text-right">Modified</span>
              <span className="w-[60px] text-right">Perms</span>
            </div>
            {files.map((file) => (
              <div
                key={file.path}
                onClick={() => handleFileClick(file)}
                onContextMenu={(e) => handleContextMenu(e, file)}
                className="flex items-center px-3 py-1.5 hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors border-b border-[var(--border)]/50 group"
              >
                <div className="flex items-center gap-2 flex-1 min-w-0">
                  {file.is_dir ? (
                    <Folder size={14} className="flex-shrink-0 text-[var(--accent)]" />
                  ) : (
                    <File size={14} className="flex-shrink-0 text-[var(--text-muted)]" />
                  )}
                  <span className="truncate text-[var(--text-primary)]">{file.name}</span>
                </div>
                <span className="w-[70px] text-right text-[var(--text-muted)] flex-shrink-0">
                  {file.is_dir ? '-' : formatFileSize(file.size)}
                </span>
                <span className="w-[80px] text-right text-[var(--text-muted)] flex-shrink-0 truncate">
                  {file.modified || '-'}
                </span>
                <span className="w-[60px] text-right text-[var(--text-muted)] flex-shrink-0 font-mono">
                  {formatPermissions(file.permissions)}
                </span>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setContextMenu({ x: e.clientX, y: e.clientY, file });
                  }}
                  className="p-0.5 rounded opacity-0 group-hover:opacity-100 text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-primary)] transition-all ml-1"
                >
                  <MoreVertical size={12} />
                </button>
              </div>
            ))}
          </div>
        ) : (
          /* Grid view */
          <div className="grid grid-cols-2 gap-1.5 p-2">
            {files.map((file) => (
              <div
                key={file.path}
                onClick={() => handleFileClick(file)}
                onContextMenu={(e) => handleContextMenu(e, file)}
                className="flex flex-col items-center justify-center p-3 rounded-lg hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors border border-transparent hover:border-[var(--border)] group relative"
              >
                {file.is_dir ? (
                  <Folder size={28} className="text-[var(--accent)] mb-1.5" />
                ) : (
                  <File size={28} className="text-[var(--text-muted)] mb-1.5" />
                )}
                <span className="text-xs text-[var(--text-primary)] text-center truncate w-full px-1">
                  {file.name}
                </span>
                <span className="text-[10px] text-[var(--text-muted)] mt-0.5">
                  {file.is_dir ? '-' : formatFileSize(file.size)}
                </span>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setContextMenu({ x: e.clientX, y: e.clientY, file });
                  }}
                  className="absolute top-1 right-1 p-0.5 rounded opacity-0 group-hover:opacity-100 text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-primary)] transition-all"
                >
                  <MoreVertical size={10} />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="px-3 py-1.5 border-t border-[var(--border)] bg-[var(--bg-secondary)] text-[10px] text-[var(--text-muted)]">
        {loading ? 'Loading...' : `${files.length} item${files.length !== 1 ? 's' : ''}`}
      </div>

      {/* Context menu */}
      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          file={contextMenu.file}
          onRename={(file) => setRenameTarget(file)}
          onDelete={handleDelete}
          onDownload={handleDownload}
          onClose={() => setContextMenu(null)}
        />
      )}

      {/* Rename dialog */}
      {renameTarget && (
        <RenameDialog
          currentName={renameTarget.name}
          onConfirm={handleRename}
          onCancel={() => setRenameTarget(null)}
        />
      )}
    </div>
  );
}
