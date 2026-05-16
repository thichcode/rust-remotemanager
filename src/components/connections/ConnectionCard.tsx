import { useState } from 'react';
import {
  Star,
  StarOff,
  Plug,
  PlugZap,
  Pencil,
  Trash2,
  Wifi,
  WifiOff,
  ChevronRight,
  Terminal,
  Monitor,
  Cable,
} from 'lucide-react';
import type { Connection, ConnectionType } from '../../services/types';

interface ConnectionCardProps {
  connection: Connection;
  onConnect?: (connection: Connection) => void;
  onEdit?: (connection: Connection) => void;
  onDelete?: (connection: Connection) => void;
  onToggleFavorite?: (connection: Connection) => void;
}

const typeIcons: Record<ConnectionType, typeof Terminal> = {
  ssh: Terminal,
  rdp: Monitor,
  serial: Cable,
};

const typeColors: Record<ConnectionType, string> = {
  ssh: 'text-[var(--status-info)]',
  rdp: 'text-[var(--status-success)]',
  serial: 'text-[var(--status-warning)]',
};

const typeLabels: Record<ConnectionType, string> = {
  ssh: 'SSH',
  rdp: 'RDP',
  serial: 'Serial',
};

export default function ConnectionCard({
  connection,
  onConnect,
  onEdit,
  onDelete,
  onToggleFavorite,
}: ConnectionCardProps) {
  const [isHovered, setIsHovered] = useState(false);

  const TypeIcon = typeIcons[connection.type];
  const typeColor = typeColors[connection.type];

  return (
    <div
      className="group relative rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] hover:border-[var(--accent)]/40 hover:shadow-md hover:shadow-[var(--accent)]/5 transition-all duration-200 cursor-pointer"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      onClick={() => onConnect?.(connection)}
    >
      {/* Color indicator strip */}
      <div
        className="absolute left-0 top-0 bottom-0 w-1 rounded-l-xl"
        style={{
          backgroundColor: connection.color || 'var(--accent)',
        }}
      />

      <div className="pl-4 pr-4 py-3">
        <div className="flex items-start justify-between">
          {/* Left: icon + info */}
          <div className="flex items-start gap-3 min-w-0 flex-1">
            <div
              className={`p-2 rounded-lg flex-shrink-0 ${
                isHovered ? 'bg-[var(--accent-muted)]' : 'bg-[var(--bg-tertiary)]'
              } transition-colors`}
            >
              <TypeIcon
                size={18}
                className={`${typeColor} transition-colors`}
              />
            </div>
            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2">
                <h3 className="font-semibold text-sm text-[var(--text-primary)] truncate">
                  {connection.name}
                </h3>
                {/* Type badge */}
                <span
                  className={`text-[10px] font-medium px-1.5 py-0.5 rounded ${
                    connection.type === 'ssh'
                      ? 'bg-blue-500/10 text-blue-400'
                      : connection.type === 'rdp'
                      ? 'bg-green-500/10 text-green-400'
                      : 'bg-yellow-500/10 text-yellow-400'
                  }`}
                >
                  {typeLabels[connection.type]}
                </span>
                {/* Status dot */}
                <span className="w-1.5 h-1.5 rounded-full bg-[var(--text-muted)] flex-shrink-0" />
              </div>
              <p className="text-xs text-[var(--text-secondary)] mt-0.5 font-mono">
                {connection.username}@{connection.host}:{connection.port}
              </p>
              {/* Tags */}
              {Array.isArray(connection.tags) && connection.tags.length > 0 && (
                <div className="flex flex-wrap gap-1 mt-1.5">
                  {connection.tags.map((tag) => (
                    <span
                      key={tag}
                      className="text-[10px] px-1.5 py-0.5 rounded bg-[var(--bg-tertiary)] text-[var(--text-muted)]"
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Right: actions (visible on hover) */}
          <div className="flex items-center gap-1 flex-shrink-0 ml-2">
            {isHovered ? (
              <>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onToggleFavorite?.(connection);
                  }}
                  className="p-1.5 rounded-md text-[var(--text-muted)] hover:text-[var(--warning)] hover:bg-[var(--bg-tertiary)] transition-colors"
                  title={connection.isFavorite ? 'Remove favorite' : 'Add favorite'}
                >
                  {connection.isFavorite ? (
                    <Star size={14} className="text-[var(--warning)] fill-[var(--warning)]" />
                  ) : (
                    <StarOff size={14} />
                  )}
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onEdit?.(connection);
                  }}
                  className="p-1.5 rounded-md text-[var(--text-muted)] hover:text-[var(--accent)] hover:bg-[var(--bg-tertiary)] transition-colors"
                  title="Edit connection"
                >
                  <Pencil size={14} />
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onDelete?.(connection);
                  }}
                  className="p-1.5 rounded-md text-[var(--text-muted)] hover:text-[var(--status-error)] hover:bg-[var(--status-error)]/10 transition-colors"
                  title="Delete connection"
                >
                  <Trash2 size={14} />
                </button>
                <span className="w-px h-4 bg-[var(--border)] mx-1" />
                <ChevronRight size={14} className="text-[var(--text-muted)]" />
              </>
            ) : (
              <div className="flex items-center gap-1">
                {connection.isFavorite && (
                  <Star size={14} className="text-[var(--warning)] fill-[var(--warning)]" />
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
