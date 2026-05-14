import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Terminal,
  Wifi,
  Key,
  ArrowRight,
  Clock,
  Star,
  TrendingUp,
} from 'lucide-react';
import { useConnectionStore } from '../stores/connectionStore';
import { useSessionStore } from '../stores/sessionStore';

export default function Dashboard() {
  const navigate = useNavigate();
  const { connections } = useConnectionStore();
  const { sessions } = useSessionStore();
  const [quickHost, setQuickHost] = useState('');

  const activeSessions = sessions.filter((s) => s.state === 'connected');
  const favoriteConnections = connections.filter((c) => c.isFavorite);
  const recentConnections = [...connections]
    .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
    .slice(0, 8);

  const stats = [
    {
      label: 'Total Connections',
      value: connections.length,
      icon: Wifi,
      color: 'text-[var(--accent)]',
      bg: 'bg-[var(--accent-muted)]',
      onClick: () => navigate('/connections'),
    },
    {
      label: 'Active Sessions',
      value: activeSessions.length,
      icon: Terminal,
      color: 'text-[var(--status-success)]',
      bg: 'bg-[var(--status-success)]/10',
      subtitle:
        activeSessions.length === 0
          ? 'No active connections'
          : `${activeSessions.length} running`,
    },
    {
      label: 'Favorites',
      value: favoriteConnections.length,
      icon: Star,
      color: 'text-[var(--status-warning)]',
      bg: 'bg-[var(--status-warning)]/10',
      onClick: () => navigate('/connections'),
    },
  ];

  const handleQuickConnect = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      if (!quickHost.trim()) return;

      // Check if there's a connection matching this host
      const match = connections.find(
        (c) =>
          c.host === quickHost.trim() ||
          `${c.username}@${c.host}` === quickHost.trim(),
      );

      if (match) {
        navigate('/connections');
      } else {
        // Navigate to connections page where user can create
        navigate('/connections');
      }
      setQuickHost('');
    },
    [quickHost, connections, navigate],
  );

  return (
    <div className="p-6 space-y-8 max-w-6xl mx-auto">
      {/* Header */}
      <div className="space-y-1">
        <h1 className="text-2xl font-bold text-[var(--text-primary)] tracking-tight">
          Welcome to Hermes
        </h1>
        <p className="text-sm text-[var(--text-secondary)]">
          Remote connection manager — manage SSH, RDP, and serial connections
        </p>
      </div>

      {/* Quick Connect Bar */}
      <div className="rounded-2xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5">
        <form onSubmit={handleQuickConnect} className="flex items-center gap-3">
          <div className="flex-1 relative">
            <Wifi
              size={16}
              className="absolute left-3.5 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
            />
            <input
              type="text"
              value={quickHost}
              onChange={(e) => setQuickHost(e.target.value)}
              placeholder="Quick connect — hostname or user@host..."
              className="w-full pl-10 pr-4 py-2.5 rounded-xl border border-[var(--border)] bg-[var(--bg-primary)] text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors"
            />
          </div>
          <button
            type="submit"
            disabled={!quickHost.trim()}
            className="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl bg-[var(--accent)] text-white text-sm font-medium hover:bg-[var(--accent-hover)] disabled:opacity-50 disabled:cursor-not-allowed active:scale-[0.98] transition-all"
          >
            Connect
            <ArrowRight size={16} />
          </button>
        </form>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {stats.map((stat) => (
          <div
            key={stat.label}
            onClick={stat.onClick}
            className={`rounded-2xl border border-[var(--border)] bg-[var(--bg-secondary)] p-5 ${
              stat.onClick ? 'cursor-pointer hover:border-[var(--accent)]/40 hover:shadow-md transition-all' : ''
            }`}
          >
            <div className="flex items-start justify-between mb-3">
              <div className={`p-2.5 rounded-xl ${stat.bg}`}>
                <stat.icon size={20} className={stat.color} />
              </div>
              {stat.onClick && (
                <TrendingUp
                  size={16}
                  className="text-[var(--text-muted)] opacity-0 group-hover:opacity-100"
                />
              )}
            </div>
            <p className="text-3xl font-bold text-[var(--text-primary)]">
              {stat.value}
            </p>
            <p className="text-sm text-[var(--text-secondary)] mt-1">
              {stat.label}
            </p>
            {stat.subtitle && (
              <p className="text-xs text-[var(--text-muted)] mt-0.5">
                {stat.subtitle}
              </p>
            )}
          </div>
        ))}
      </div>

      {/* Recent Connections */}
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-bold text-[var(--text-primary)] flex items-center gap-2">
            <Clock size={18} className="text-[var(--text-muted)]" />
            Recent Connections
          </h2>
          {connections.length > 0 && (
            <button
              onClick={() => navigate('/connections')}
              className="text-sm text-[var(--accent)] hover:text-[var(--accent-hover)] transition-colors"
            >
              View all
            </button>
          )}
        </div>

        {recentConnections.length === 0 ? (
          <div className="rounded-2xl border border-[var(--border)] bg-[var(--bg-secondary)] p-8 text-center">
            <Wifi size={40} className="mx-auto mb-3 text-[var(--text-muted)] opacity-30" />
            <p className="text-base font-medium text-[var(--text-primary)]">
              No connections yet
            </p>
            <p className="text-sm text-[var(--text-secondary)] mt-1">
              Create your first connection to get started
            </p>
            <button
              onClick={() => navigate('/connections')}
              className="mt-4 inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-[var(--accent)] text-white text-sm font-medium hover:bg-[var(--accent-hover)] transition-colors"
            >
              <Terminal size={16} />
              New Connection
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
            {recentConnections.map((conn) => (
              <div
                key={conn.id}
                onClick={() => navigate('/connections')}
                className="rounded-xl border border-[var(--border)] bg-[var(--bg-secondary)] p-4 cursor-pointer hover:border-[var(--accent)]/40 hover:shadow-md transition-all"
              >
                <div className="flex items-center gap-2 mb-2">
                  <div className="w-2 h-2 rounded-full bg-[var(--status-success)]" />
                  <span className="text-xs font-medium uppercase text-[var(--text-muted)]">
                    {conn.type.toUpperCase()}
                  </span>
                  {conn.isFavorite && (
                    <Star
                      size={12}
                      className="text-[var(--warning)] fill-[var(--warning)] ml-auto"
                    />
                  )}
                </div>
                <p className="text-sm font-semibold text-[var(--text-primary)] truncate">
                  {conn.name}
                </p>
                <p className="text-xs text-[var(--text-muted)] mt-0.5 font-mono">
                  {conn.username}@{conn.host}:{conn.port}
                </p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
