import { useEffect, useState } from 'react';
import {
  Wifi,
  Lock,
  Unlock,
  Moon,
  Sun,
  Globe,
} from 'lucide-react';
import { useTheme } from '../../themes/theme';
import { useSessionStore } from '../../stores/sessionStore';
import { vaultStatus } from '../../services/ipc';

export default function StatusBar() {
  const { isDark, toggle: toggleTheme } = useTheme();
  const { sessions } = useSessionStore();
  const [vaultLocked, setVaultLocked] = useState<boolean | null>(null);

  const activeCount = sessions.filter((s) => s.state === 'connected').length;

  useEffect(() => {
    const checkVault = async () => {
      try {
        const result = await vaultStatus();
        if (result.success && result.data) {
          setVaultLocked(result.data);
        }
      } catch {
        // Vault not available
        setVaultLocked(null);
      }
    };
    checkVault();

    const interval = setInterval(checkVault, 30000);
    return () => clearInterval(interval);
  }, []);

  return (
    <footer className="flex items-center justify-between h-8 px-4 text-xs border-t border-[var(--border)] bg-[var(--bg-secondary)] text-[var(--text-muted)] flex-shrink-0">
      {/* Left: active connections count */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-1.5">
          <Wifi size={12} />
          <span>
            {activeCount} active {activeCount === 1 ? 'session' : 'sessions'}
          </span>
        </div>
        <span className="w-px h-3 bg-[var(--border)]" />
        <div className="flex items-center gap-1.5">
          <Globe size={12} />
          <span>{sessions.length} total</span>
        </div>
      </div>

      {/* Center: vault status */}
      <div className="flex items-center gap-1.5">
        {vaultLocked === true && (
          <>
            <Lock size={12} className="text-[var(--warning)]" />
            <span className="text-[var(--warning)]">Vault locked</span>
          </>
        )}
        {vaultLocked === false && (
          <>
            <Unlock size={12} className="text-[var(--success)]" />
            <span className="text-[var(--success)]">Vault unlocked</span>
          </>
        )}
        {vaultLocked === null && (
          <span>Vault unavailable</span>
        )}
      </div>

      {/* Right: theme toggle */}
      <button
        onClick={toggleTheme}
        className="flex items-center gap-1.5 hover:text-[var(--text-primary)] transition-colors"
        aria-label={`Switch to ${isDark ? 'light' : 'dark'} mode`}
      >
        {isDark ? <Sun size={12} /> : <Moon size={12} />}
        <span>{isDark ? 'Light' : 'Dark'}</span>
      </button>
    </footer>
  );
}
