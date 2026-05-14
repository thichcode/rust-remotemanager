import { useLocation, NavLink } from 'react-router-dom';
import {
  LayoutDashboard,
  Server,
  Settings,
  ChevronLeft,
  ChevronRight,
  Terminal,
  Wifi,
  Lock,
  Unlock,
} from 'lucide-react';
import { useTheme } from '../../themes/theme';
import { useUiStore } from '../../stores/uiStore';
import { useSessionStore } from '../../stores/sessionStore';

const navItems = [
  { label: 'Dashboard', path: '/', icon: LayoutDashboard },
  { label: 'Connections', path: '/connections', icon: Server },
  { label: 'Settings', path: '/settings', icon: Settings },
];

export default function Sidebar() {
  const { isDark, toggle: toggleTheme } = useTheme();
  const location = useLocation();
  const { sidebarCollapsed, toggleSidebar } = useUiStore();
  const { sessions } = useSessionStore();

  const activeSessions = sessions.filter((s) => s.state === 'connected');

  return (
    <aside
      className={`flex flex-col border-r border-[var(--border)] bg-[var(--bg-secondary)] transition-all duration-200 ease-in-out ${
        sidebarCollapsed ? 'w-[var(--sidebar-collapsed-width)]' : 'w-[var(--sidebar-width)]'
      } flex-shrink-0`}
    >
      {/* Header / Brand */}
      <div className="flex items-center h-[var(--header-height)] px-4 border-b border-[var(--border)]">
        <div className="flex items-center gap-3 overflow-hidden">
          <div className="flex-shrink-0 w-8 h-8 rounded-lg bg-[var(--accent)] flex items-center justify-center">
            <Terminal size={18} className="text-white" />
          </div>
          {!sidebarCollapsed && (
            <span className="font-bold text-[var(--text-primary)] whitespace-nowrap text-base tracking-tight">
              Hermes
            </span>
          )}
        </div>
        <button
          onClick={toggleSidebar}
          className="ml-auto p-1.5 rounded-md text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
          aria-label={sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {sidebarCollapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
        </button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 py-3 space-y-0.5 px-2 overflow-y-auto">
        {navItems.map((item) => {
          const isActive = location.pathname === item.path;
          return (
            <NavLink
              key={item.path}
              to={item.path}
              className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors duration-150 ${
                isActive
                  ? 'bg-[var(--accent-muted)] text-[var(--accent)]'
                  : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
              }`}
              title={sidebarCollapsed ? item.label : undefined}
            >
              <item.icon size={20} className="flex-shrink-0" />
              {!sidebarCollapsed && <span>{item.label}</span>}
            </NavLink>
          );
        })}
      </nav>

      {/* Active Sessions (bottom area) */}
      {!sidebarCollapsed && sessions.length > 0 && (
        <div className="px-3 py-2 border-t border-[var(--border)]">
          <p className="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-2 px-1">
            Sessions ({activeSessions.length}/{sessions.length})
          </p>
          <div className="space-y-0.5 max-h-[180px] overflow-y-auto">
            {sessions.map((session) => {
              const isConnected = session.state === 'connected';
              return (
                <div
                  key={session.id}
                  className="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors"
                >
                  <span
                    className={`w-1.5 h-1.5 rounded-full flex-shrink-0 ${
                      isConnected
                        ? 'bg-[var(--success)]'
                        : session.state === 'connecting'
                        ? 'bg-[var(--warning)] animate-pulse'
                        : 'bg-[var(--text-muted)]'
                    }`}
                  />
                  <span className="truncate">
                    Session {session.id.slice(0, 8)}
                  </span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Theme toggle */}
      <div className="p-2 border-t border-[var(--border)]">
        <button
          onClick={toggleTheme}
          className={`flex items-center gap-3 w-full px-3 py-2.5 rounded-lg text-sm font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors`}
          title={sidebarCollapsed ? (isDark ? 'Light Mode' : 'Dark Mode') : undefined}
        >
          {isDark ? (
            <Wifi size={20} className="flex-shrink-0" />
          ) : (
            <Lock size={20} className="flex-shrink-0" />
          )}
          {!sidebarCollapsed && <span>{isDark ? 'Status' : 'Status'}</span>}
        </button>
      </div>
    </aside>
  );
}
