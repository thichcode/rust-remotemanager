import { useState, useMemo } from 'react';
import {
  Search,
  SlidersHorizontal,
  Grid3X3,
  List,
  Terminal,
  Monitor,
  Cable,
  X,
} from 'lucide-react';
import ConnectionCard from './ConnectionCard';
import type { Connection, ConnectionType } from '../../services/types';

interface ConnectionListProps {
  connections: Connection[];
  searchTerm: string;
  onSearchChange: (term: string) => void;
  filterType: ConnectionType | 'all';
  onFilterTypeChange: (type: ConnectionType | 'all') => void;
  onConnect?: (connection: Connection) => void;
  onEdit?: (connection: Connection) => void;
  onDelete?: (connection: Connection) => void;
  onToggleFavorite?: (connection: Connection) => void;
}

const typeFilters: { value: ConnectionType | 'all'; label: string; icon: typeof Terminal }[] = [
  { value: 'all', label: 'All', icon: Terminal },
  { value: 'ssh', label: 'SSH', icon: Terminal },
  { value: 'rdp', label: 'RDP', icon: Monitor },
  { value: 'serial', label: 'Serial', icon: Cable },
];

export default function ConnectionList({
  connections,
  searchTerm,
  onSearchChange,
  filterType,
  onFilterTypeChange,
  onConnect,
  onEdit,
  onDelete,
  onToggleFavorite,
}: ConnectionListProps) {
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');

  const filteredConnections = useMemo(() => {
    let result = connections;

    // Filter by type
    if (filterType !== 'all') {
      result = result.filter((c) => c.type === filterType);
    }

    // Filter by search
    if (searchTerm.trim()) {
      const term = searchTerm.toLowerCase();
      result = result.filter(
        (c) =>
          c.name.toLowerCase().includes(term) ||
          c.host.toLowerCase().includes(term) ||
          c.username.toLowerCase().includes(term) ||
          (c.tags && c.tags.some((t) => t.toLowerCase().includes(term))),
      );
    }

    return result;
  }, [connections, filterType, searchTerm]);

  return (
    <div className="space-y-4">
      {/* Search and filter bar */}
      <div className="flex items-center gap-3">
        {/* Search */}
        <div className="relative flex-1 max-w-md">
          <Search
            size={16}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
          />
          <input
            type="text"
            placeholder="Search connections by name, host, or tag..."
            value={searchTerm}
            onChange={(e) => onSearchChange(e.target.value)}
            className="w-full pl-9 pr-8 py-2 rounded-lg border border-[var(--border)] bg-[var(--bg-secondary)] text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors"
          />
          {searchTerm && (
            <button
              onClick={() => onSearchChange('')}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)] hover:text-[var(--text-primary)]"
            >
              <X size={14} />
            </button>
          )}
        </div>

        {/* Type filters */}
        <div className="flex items-center gap-1 bg-[var(--bg-secondary)] rounded-lg border border-[var(--border)] p-0.5">
          {typeFilters.map((f) => (
            <button
              key={f.value}
              onClick={() => onFilterTypeChange(f.value)}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
                filterType === f.value
                  ? 'bg-[var(--accent)] text-white'
                  : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
              }`}
            >
              <f.icon size={14} />
              {f.label}
            </button>
          ))}
        </div>

        {/* View toggle */}
        <div className="flex items-center gap-0.5 bg-[var(--bg-secondary)] rounded-lg border border-[var(--border)] p-0.5">
          <button
            onClick={() => setViewMode('grid')}
            className={`p-1.5 rounded-md transition-colors ${
              viewMode === 'grid'
                ? 'bg-[var(--accent)] text-white'
                : 'text-[var(--text-muted)] hover:text-[var(--text-primary)]'
            }`}
            title="Grid view"
          >
            <Grid3X3 size={14} />
          </button>
          <button
            onClick={() => setViewMode('list')}
            className={`p-1.5 rounded-md transition-colors ${
              viewMode === 'list'
                ? 'bg-[var(--accent)] text-white'
                : 'text-[var(--text-muted)] hover:text-[var(--text-primary)]'
            }`}
            title="List view"
          >
            <List size={14} />
          </button>
        </div>

        {/* Count */}
        <span className="text-xs text-[var(--text-muted)] whitespace-nowrap">
          {filteredConnections.length} of {connections.length}
        </span>
      </div>

      {/* Connection cards */}
      {filteredConnections.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-[var(--text-muted)]">
          <Search size={48} className="opacity-20 mb-4" />
          <p className="text-base font-medium">
            {searchTerm ? 'No connections match your search' : 'No connections yet'}
          </p>
          <p className="text-sm mt-1">
            {searchTerm
              ? 'Try a different search term or filter'
              : 'Create your first connection to get started'}
          </p>
        </div>
      ) : (
        <div
          className={
            viewMode === 'grid'
              ? 'grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-3'
              : 'space-y-2'
          }
        >
          {filteredConnections.map((conn) => (
            <ConnectionCard
              key={conn.id}
              connection={conn}
              onConnect={onConnect}
              onEdit={onEdit}
              onDelete={onDelete}
              onToggleFavorite={onToggleFavorite}
            />
          ))}
        </div>
      )}
    </div>
  );
}
