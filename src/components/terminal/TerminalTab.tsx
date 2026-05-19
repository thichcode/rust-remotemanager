import { useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  X,
  Plus,
  Wifi,
  WifiOff,
  Loader2,
  AlertCircle,
} from 'lucide-react';
import { useSessionStore } from '../../stores/sessionStore';
import { useConnectionStore } from '../../stores/connectionStore';
import { disconnectSession } from '../../services/ipc';

interface TerminalTabProps {
  onNewConnection: () => void;
}

export default function TerminalTab({ onNewConnection }: TerminalTabProps) {
  const { sessions, activeSessionId, setActiveSession, removeSession } =
    useSessionStore();
  const { connections } = useConnectionStore();
  const navigate = useNavigate();

  const getConnectionName = useCallback(
    (sessionId: string, connectionId: string) => {
      const conn = connections.find((c) => c.id === connectionId);
      const baseName = conn?.name ?? `Session ${connectionId.slice(0, 8)}`;
      const sameConnSessions = sessions.filter(
        (s) => s.connectionId === connectionId,
      );
      if (sameConnSessions.length > 1) {
        const idx = sameConnSessions.findIndex((s) => s.id === sessionId) + 1;
        return idx > 1 ? `${baseName} (${idx})` : baseName;
      }
      return baseName;
    },
    [connections, sessions],
  );

  const handleTabClick = (sessionId: string) => {
    setActiveSession(sessionId);
    navigate(`/terminal/${sessionId}`);
  };

  const handleCloseTab = async (e: React.MouseEvent, sessionId: string) => {
    e.stopPropagation();
    await disconnectSession(sessionId);
    removeSession(sessionId);
  };

  const stateIcons = {
    connecting: Loader2,
    connected: Wifi,
    disconnected: WifiOff,
    error: AlertCircle,
  };

  const stateColors = {
    connecting: 'text-[var(--warning)] animate-spin',
    connected: 'text-[var(--success)]',
    disconnected: 'text-[var(--text-muted)]',
    error: 'text-[var(--status-error)]',
  };

  return (
    <div className="flex items-center bg-[var(--bg-tertiary)] border-b border-[var(--border)] overflow-x-auto flex-shrink-0">
      {/* Session tabs */}
      {sessions.map((session) => {
        const Icon = stateIcons[session.state];
        const color = stateColors[session.state];
        const isActive = session.id === activeSessionId;
        const name = getConnectionName(session.id, session.connectionId);

        return (
          <div
            key={session.id}
            onClick={() => handleTabClick(session.id)}
            className={`group flex items-center gap-2 px-3 py-2 text-xs font-medium border-r border-[var(--border)] cursor-pointer select-none transition-colors min-w-0 max-w-[160px] ${
              isActive
                ? 'bg-[var(--bg-primary)] text-[var(--text-primary)]'
                : 'bg-transparent text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-primary)]/50'
            }`}
          >
            <Icon size={12} className={`flex-shrink-0 ${color}`} />
            <span className="truncate">{name}</span>
            <button
              onClick={(e) => handleCloseTab(e, session.id)}
              className="p-0.5 rounded flex-shrink-0 opacity-0 group-hover:opacity-100 hover:bg-[var(--bg-tertiary)] hover:text-[var(--status-error)] transition-all"
            >
              <X size={12} />
            </button>
          </div>
        );
      })}

      {/* New connection button */}
      <button
        onClick={onNewConnection}
        className="flex items-center gap-1.5 px-3 py-2 text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-primary)]/50 transition-colors"
        title="New connection"
      >
        <Plus size={14} />
        <span className="hidden sm:inline">New</span>
      </button>

      {/* Empty state */}
      {sessions.length === 0 && (
        <div className="flex-1 flex items-center justify-center px-4 py-1.5">
          <span className="text-xs text-[var(--text-muted)]">
            No active sessions — click + to connect
          </span>
        </div>
      )}
    </div>
  );
}
