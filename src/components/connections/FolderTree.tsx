import { useState, useCallback, type DragEvent } from 'react';
import {
  Folder,
  FolderOpen,
  ChevronRight,
  ChevronDown,
  Plus,
  Pencil,
  Trash2,
  FolderPlus,
} from 'lucide-react';
import type { Folder as FolderType, Connection } from '../../services/types';

interface FolderTreeProps {
  folders: FolderType[];
  connections: Connection[];
  selectedFolderId: string | null;
  onSelectFolder: (id: string | null) => void;
  onNewFolder: (parentId?: string) => void;
  onEditFolder: (folder: FolderType) => void;
  onDeleteFolder: (folder: FolderType) => void;
  onMoveConnection?: (connectionId: string, folderId: string | null) => void;
}

interface TreeNode {
  folder: FolderType;
  children: TreeNode[];
  depth: number;
}

function buildTree(folders: FolderType[], parentId?: string, depth = 0): TreeNode[] {
  return folders
    .filter((f) => f.parentId === parentId)
    .sort((a, b) => a.sortOrder - b.sortOrder)
    .map((folder) => ({
      folder,
      children: buildTree(folders, folder.id, depth + 1),
      depth,
    }));
}

function FolderTreeNode({
  node,
  connections,
  selectedFolderId,
  onSelectFolder,
  onNewFolder,
  onEditFolder,
  onDeleteFolder,
  onDrop,
  expandedIds,
  toggleExpanded,
}: {
  node: TreeNode;
  connections: Connection[];
  selectedFolderId: string | null;
  onSelectFolder: (id: string | null) => void;
  onNewFolder: (parentId?: string) => void;
  onEditFolder: (folder: FolderType) => void;
  onDeleteFolder: (folder: FolderType) => void;
  onDrop: (targetFolderId: string | null) => void;
  expandedIds: Set<string>;
  toggleExpanded: (id: string) => void;
}) {
  const [isHovered, setIsHovered] = useState(false);
  const [isDragOver, setIsDragOver] = useState(false);
  const isExpanded = expandedIds.has(node.folder.id);
  const isSelected = selectedFolderId === node.folder.id;
  const connCount = connections.filter(
    (c) => c.folderId === node.folder.id,
  ).length;

  const handleDragOver = useCallback(
    (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragOver(true);
    },
    [],
  );

  const handleDragLeave = useCallback(() => {
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback(
    (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragOver(false);
      onDrop(node.folder.id);
    },
    [node.folder.id, onDrop],
  );

  return (
    <div>
      <div
        className={`flex items-center gap-1 px-2 py-1.5 rounded-md cursor-pointer transition-colors group ${
          isSelected
            ? 'bg-[var(--accent-muted)] text-[var(--accent)]'
            : isDragOver
            ? 'bg-[var(--accent)]/20 text-[var(--text-primary)]'
            : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
        }`}
        style={{ paddingLeft: `${12 + node.depth * 16}px` }}
        onClick={() => {
          onSelectFolder(node.folder.id);
          toggleExpanded(node.folder.id);
        }}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        {/* Expand/collapse */}
        <button
          onClick={(e) => {
            e.stopPropagation();
            toggleExpanded(node.folder.id);
          }}
          className="p-0.5 text-[var(--text-muted)] hover:text-[var(--text-primary)]"
        >
          {node.children.length > 0 ? (
            isExpanded ? (
              <ChevronDown size={14} />
            ) : (
              <ChevronRight size={14} />
            )
          ) : (
            <span className="w-[14px]" />
          )}
        </button>

        {/* Folder icon */}
        {isExpanded ? (
          <FolderOpen size={16} className="text-[var(--warning)]" />
        ) : (
          <Folder size={16} className="text-[var(--warning)]" />
        )}

        {/* Name */}
        <span className="text-sm truncate flex-1">{node.folder.name}</span>

        {/* Count */}
        <span className="text-xs text-[var(--text-muted)]">{connCount}</span>

        {/* Actions */}
        {isHovered && (
          <div className="flex items-center gap-0.5">
            <button
              onClick={(e) => {
                e.stopPropagation();
                onNewFolder(node.folder.id);
              }}
              className="p-0.5 rounded text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]"
              title="New subfolder"
            >
              <FolderPlus size={12} />
            </button>
            <button
              onClick={(e) => {
                e.stopPropagation();
                onEditFolder(node.folder);
              }}
              className="p-0.5 rounded text-[var(--text-muted)] hover:text-[var(--accent)] hover:bg-[var(--bg-tertiary)]"
              title="Rename folder"
            >
              <Pencil size={12} />
            </button>
            <button
              onClick={(e) => {
                e.stopPropagation();
                onDeleteFolder(node.folder);
              }}
              className="p-0.5 rounded text-[var(--text-muted)] hover:text-[var(--status-error)] hover:bg-[var(--status-error)]/10"
              title="Delete folder"
            >
              <Trash2 size={12} />
            </button>
          </div>
        )}
      </div>

      {/* Children */}
      {isExpanded && node.children.length > 0 && (
        <div>
          {node.children.map((child) => (
            <FolderTreeNode
              key={child.folder.id}
              node={child}
              connections={connections}
              selectedFolderId={selectedFolderId}
              onSelectFolder={onSelectFolder}
              onNewFolder={onNewFolder}
              onEditFolder={onEditFolder}
              onDeleteFolder={onDeleteFolder}
              onDrop={onDrop}
              expandedIds={expandedIds}
              toggleExpanded={toggleExpanded}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export default function FolderTree({
  folders,
  connections,
  selectedFolderId,
  onSelectFolder,
  onNewFolder,
  onEditFolder,
  onDeleteFolder,
  onMoveConnection,
}: FolderTreeProps) {
  const [expandedIds, setExpandedIds] = useState<Set<string>>(() => {
    // Expand all by default
    return new Set(folders.map((f) => f.id));
  });

  const toggleExpanded = useCallback((id: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const tree = buildTree(folders);

  const handleDrop = useCallback(
    (targetFolderId: string | null) => {
      // This would need the dragged connection ID - in a real app,
      // we'd track drag state at a higher level
    },
    [],
  );

  const rootConnCount = connections.filter((c) => !c.folderId).length;

  return (
    <div className="space-y-1">
      {/* "All Connections" root */}
      <div
        className={`flex items-center gap-2 px-3 py-2 rounded-md cursor-pointer transition-colors ${
          selectedFolderId === null
            ? 'bg-[var(--accent-muted)] text-[var(--accent)]'
            : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
        }`}
        onClick={() => onSelectFolder(null)}
      >
        <FolderOpen size={16} />
        <span className="text-sm font-medium flex-1">All Connections</span>
        <span className="text-xs text-[var(--text-muted)]">{connections.length}</span>
      </div>

      {/* Folders */}
      <div className="space-y-0.5 ml-1">
        {tree.map((node) => (
          <FolderTreeNode
            key={node.folder.id}
            node={node}
            connections={connections}
            selectedFolderId={selectedFolderId}
            onSelectFolder={onSelectFolder}
            onNewFolder={onNewFolder}
            onEditFolder={onEditFolder}
            onDeleteFolder={onDeleteFolder}
            onDrop={handleDrop}
            expandedIds={expandedIds}
            toggleExpanded={toggleExpanded}
          />
        ))}
      </div>

      {/* Empty state */}
      {folders.length === 0 && (
        <div className="px-3 py-4 text-center text-xs text-[var(--text-muted)]">
          <Folder size={24} className="mx-auto mb-2 opacity-30" />
          <p>No folders yet</p>
          <button
            onClick={() => onNewFolder()}
            className="mt-2 inline-flex items-center gap-1 text-[var(--accent)] hover:text-[var(--accent-hover)]"
          >
            <Plus size={12} />
            Create folder
          </button>
        </div>
      )}
    </div>
  );
}
