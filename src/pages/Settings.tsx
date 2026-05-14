import { useState } from 'react';
import {
  Settings2,
  Palette,
  Shield,
  Terminal,
  Info,
  Sun,
  Moon,
  Lock,
  Unlock,
  Eye,
  EyeOff,
  Check,
} from 'lucide-react';
import { useTheme } from '../themes/theme';
import { vaultLock, vaultUnlock } from '../services/ipc';
import toast from 'react-hot-toast';

type SettingsTab = 'general' | 'appearance' | 'security' | 'terminal' | 'about';

const tabs: { id: SettingsTab; label: string; icon: typeof Settings2 }[] = [
  { id: 'general', label: 'General', icon: Settings2 },
  { id: 'appearance', label: 'Appearance', icon: Palette },
  { id: 'security', label: 'Security', icon: Shield },
  { id: 'terminal', label: 'Terminal', icon: Terminal },
  { id: 'about', label: 'About', icon: Info },
];

export default function Settings() {
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');
  const { mode, setMode, isDark } = useTheme();

  const [fontSize, setFontSize] = useState(14);
  const [fontFamily, setFontFamily] = useState("'JetBrains Mono', 'Fira Code', monospace");
  const [cursorStyle, setCursorStyle] = useState<'block' | 'underline' | 'bar'>('block');
  const [scrollback, setScrollback] = useState(5000);
  const [cursorBlink, setCursorBlink] = useState(true);
  const [bellStyle, setBellStyle] = useState<'none' | 'sound'>('none');

  const [vaultPassword, setVaultPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [vaultLocked, setVaultLocked] = useState(true);

  const handleVaultToggle = async () => {
    if (vaultLocked) {
      if (!vaultPassword) {
        toast.error('Please enter a master password');
        return;
      }
      const result = await vaultUnlock(vaultPassword);
      if (result.success) {
        setVaultLocked(false);
        toast.success('Vault unlocked');
        setVaultPassword('');
      } else {
        toast.error(result.error ?? 'Failed to unlock vault');
      }
    } else {
      const result = await vaultLock();
      if (result.success) {
        setVaultLocked(true);
        toast.success('Vault locked');
      } else {
        toast.error(result.error ?? 'Failed to lock vault');
      }
    }
  };

  const renderTabContent = () => {
    switch (activeTab) {
      case 'general':
        return (
          <div className="space-y-6">
            <div className="card space-y-4">
              <h3 className="text-base font-bold text-[var(--text-primary)]">
                Default Connection Settings
              </h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                    Default Keepalive (seconds)
                  </label>
                  <input
                    type="number"
                    defaultValue={30}
                    min={0}
                    className="input"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                    Default Port (SSH)
                  </label>
                  <input
                    type="number"
                    defaultValue={22}
                    min={1}
                    max={65535}
                    className="input"
                  />
                </div>
              </div>
            </div>

            <div className="card space-y-4">
              <h3 className="text-base font-bold text-[var(--text-primary)]">
                Theme Default
              </h3>
              <div className="flex items-center gap-3">
                <button
                  onClick={() => setMode('dark')}
                  className={`flex items-center gap-2 px-4 py-2 rounded-lg border text-sm font-medium transition-colors ${
                    mode === 'dark'
                      ? 'border-[var(--accent)] bg-[var(--accent-muted)] text-[var(--accent)]'
                      : 'border-[var(--border)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                  }`}
                >
                  <Moon size={16} />
                  Dark
                </button>
                <button
                  onClick={() => setMode('light')}
                  className={`flex items-center gap-2 px-4 py-2 rounded-lg border text-sm font-medium transition-colors ${
                    mode === 'light'
                      ? 'border-[var(--accent)] bg-[var(--accent-muted)] text-[var(--accent)]'
                      : 'border-[var(--border)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                  }`}
                >
                  <Sun size={16} />
                  Light
                </button>
              </div>
            </div>
          </div>
        );

      case 'appearance':
        return (
          <div className="space-y-6">
            <div className="card space-y-4">
              <h3 className="text-base font-bold text-[var(--text-primary)]">
                Theme
              </h3>
              <div className="flex items-center gap-3">
                <button
                  onClick={() => setMode('dark')}
                  className={`flex items-center gap-2 px-5 py-3 rounded-xl border text-sm font-medium transition-colors ${
                    mode === 'dark'
                      ? 'border-[var(--accent)] bg-[var(--accent-muted)] text-[var(--accent)] ring-1 ring-[var(--accent)]'
                      : 'border-[var(--border)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                  }`}
                >
                  <Moon size={18} />
                  <div className="text-left">
                    <p className="font-medium">Dark</p>
                    <p className="text-[11px] opacity-60">Easy on the eyes</p>
                  </div>
                  {mode === 'dark' && <Check size={16} className="ml-auto" />}
                </button>
                <button
                  onClick={() => setMode('light')}
                  className={`flex items-center gap-2 px-5 py-3 rounded-xl border text-sm font-medium transition-colors flex-1 ${
                    mode === 'light'
                      ? 'border-[var(--accent)] bg-[var(--accent-muted)] text-[var(--accent)] ring-1 ring-[var(--accent)]'
                      : 'border-[var(--border)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                  }`}
                >
                  <Sun size={18} />
                  <div className="text-left">
                    <p className="font-medium">Light</p>
                    <p className="text-[11px] opacity-60">Bright and clean</p>
                  </div>
                  {mode === 'light' && <Check size={16} className="ml-auto" />}
                </button>
              </div>
            </div>

            <div className="card space-y-4">
              <h3 className="text-base font-bold text-[var(--text-primary)]">
                Font Settings
              </h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                    Font Family
                  </label>
                  <select
                    value={fontFamily}
                    onChange={(e) => setFontFamily(e.target.value)}
                    className="input"
                  >
                    <option value="'JetBrains Mono', 'Fira Code', monospace">
                      JetBrains Mono
                    </option>
                    <option value="'Fira Code', monospace">Fira Code</option>
                    <option value="'Cascadia Code', monospace">
                      Cascadia Code
                    </option>
                    <option value="'Source Code Pro', monospace">
                      Source Code Pro
                    </option>
                    <option value="monospace">Default Monospace</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                    Font Size
                  </label>
                  <input
                    type="number"
                    value={fontSize}
                    onChange={(e) => setFontSize(Number(e.target.value))}
                    min={8}
                    max={32}
                    className="input"
                  />
                </div>
              </div>
            </div>
          </div>
        );

      case 'security':
        return (
          <div className="space-y-6">
            <div className="card space-y-4">
              <h3 className="text-base font-bold text-[var(--text-primary)]">
                Credential Vault
              </h3>
              <p className="text-sm text-[var(--text-secondary)]">
                The vault securely stores your SSH keys and passwords. Lock it
                when you're away from your computer.
              </p>
              <div className="flex items-center gap-3 p-3 rounded-lg bg-[var(--bg-tertiary)]">
                <div
                  className={`p-2 rounded-lg ${
                    vaultLocked ? 'bg-[var(--warning)]/10' : 'bg-[var(--success)]/10'
                  }`}
                >
                  {vaultLocked ? (
                    <Lock
                      size={20}
                      className="text-[var(--warning)]"
                    />
                  ) : (
                    <Unlock
                      size={20}
                      className="text-[var(--success)]"
                    />
                  )}
                </div>
                <div className="flex-1">
                  <p className="text-sm font-medium text-[var(--text-primary)]">
                    {vaultLocked ? 'Vault is locked' : 'Vault is unlocked'}
                  </p>
                  <p className="text-xs text-[var(--text-muted)]">
                    {vaultLocked
                      ? 'Credentials are encrypted and stored securely'
                      : 'Credentials are accessible for connections'}
                  </p>
                </div>
              </div>

              {vaultLocked && (
                <div className="space-y-3">
                  <div className="relative">
                    <input
                      type={showPassword ? 'text' : 'password'}
                      value={vaultPassword}
                      onChange={(e) => setVaultPassword(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') handleVaultToggle();
                      }}
                      placeholder="Enter master password..."
                      className="input pr-20"
                    />
                    <button
                      onClick={() => setShowPassword(!showPassword)}
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)] hover:text-[var(--text-primary)]"
                    >
                      {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
                    </button>
                  </div>
                  <button
                    onClick={handleVaultToggle}
                    className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--accent)] text-white text-sm font-medium hover:bg-[var(--accent-hover)] transition-colors"
                  >
                    <Unlock size={16} />
                    Unlock Vault
                  </button>
                </div>
              )}

              {!vaultLocked && (
                <button
                  onClick={handleVaultToggle}
                  className="inline-flex items-center gap-2 px-4 py-2 rounded-lg border border-[var(--status-error)]/30 text-[var(--status-error)] text-sm font-medium hover:bg-[var(--status-error)]/10 transition-colors"
                >
                  <Lock size={16} />
                  Lock Vault
                </button>
              )}
            </div>

            <div className="card space-y-4">
              <h3 className="text-base font-bold text-[var(--text-primary)]">
                Master Password
              </h3>
              <p className="text-sm text-[var(--text-secondary)]">
                Set or change your vault master password.
              </p>
              <button
                className="px-4 py-2 rounded-lg border border-[var(--border)] text-sm font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
                disabled
              >
                Change Password
              </button>
              <p className="text-xs text-[var(--text-muted)]">
                Coming soon
              </p>
            </div>
          </div>
        );

      case 'terminal':
        return (
          <div className="space-y-6">
            <div className="card space-y-4">
              <h3 className="text-base font-bold text-[var(--text-primary)]">
                Terminal Appearance
              </h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                    Font Family
                  </label>
                  <select
                    value={fontFamily}
                    onChange={(e) => setFontFamily(e.target.value)}
                    className="input"
                  >
                    <option value="'JetBrains Mono', 'Fira Code', monospace">
                      JetBrains Mono
                    </option>
                    <option value="'Fira Code', monospace">Fira Code</option>
                    <option value="'Cascadia Code', monospace">
                      Cascadia Code
                    </option>
                    <option value="'Source Code Pro', monospace">
                      Source Code Pro
                    </option>
                    <option value="monospace">Default Monospace</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                    Font Size
                  </label>
                  <input
                    type="number"
                    value={fontSize}
                    onChange={(e) => setFontSize(Number(e.target.value))}
                    min={8}
                    max={32}
                    className="input"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                    Cursor Style
                  </label>
                  <select
                    value={cursorStyle}
                    onChange={(e) =>
                      setCursorStyle(e.target.value as 'block' | 'underline' | 'bar')
                    }
                    className="input"
                  >
                    <option value="block">Block</option>
                    <option value="underline">Underline</option>
                    <option value="bar">Bar</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                    Scrollback (lines)
                  </label>
                  <input
                    type="number"
                    value={scrollback}
                    onChange={(e) => setScrollback(Number(e.target.value))}
                    min={100}
                    max={50000}
                    step={100}
                    className="input"
                  />
                </div>
              </div>
            </div>

            <div className="card space-y-4">
              <h3 className="text-base font-bold text-[var(--text-primary)]">
                Terminal Behavior
              </h3>
              <div className="space-y-3">
                <label className="flex items-center gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={cursorBlink}
                    onChange={(e) => setCursorBlink(e.target.checked)}
                    className="w-4 h-4 rounded border-[var(--border)] bg-[var(--bg-primary)] text-[var(--accent)] focus:ring-[var(--accent)] focus:ring-offset-0"
                  />
                  <div>
                    <p className="text-sm font-medium text-[var(--text-primary)]">
                      Blinking Cursor
                    </p>
                    <p className="text-xs text-[var(--text-muted)]">
                      Make the cursor blink
                    </p>
                  </div>
                </label>

                <label className="flex items-center gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    defaultChecked
                    className="w-4 h-4 rounded border-[var(--border)] bg-[var(--bg-primary)] text-[var(--accent)] focus:ring-[var(--accent)] focus:ring-offset-0"
                  />
                  <div>
                    <p className="text-sm font-medium text-[var(--text-primary)]">
                      Copy on Select
                    </p>
                    <p className="text-xs text-[var(--text-muted)]">
                      Automatically copy text when selecting with mouse
                    </p>
                  </div>
                </label>

                <label className="flex items-center gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    defaultChecked
                    className="w-4 h-4 rounded border-[var(--border)] bg-[var(--bg-primary)] text-[var(--accent)] focus:ring-[var(--accent)] focus:ring-offset-0"
                  />
                  <div>
                    <p className="text-sm font-medium text-[var(--text-primary)]">
                      Paste on Right Click
                    </p>
                    <p className="text-xs text-[var(--text-muted)]">
                      Paste clipboard content on right-click
                    </p>
                  </div>
                </label>

                <div>
                  <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                    Terminal Bell
                  </label>
                  <select
                    value={bellStyle}
                    onChange={(e) =>
                      setBellStyle(e.target.value as 'none' | 'sound')
                    }
                    className="input max-w-xs"
                  >
                    <option value="none">Disabled</option>
                    <option value="sound">Sound</option>
                  </select>
                </div>
              </div>
            </div>
          </div>
        );

      case 'about':
        return (
          <div className="space-y-6">
            <div className="card space-y-4 text-center py-8">
              <div className="w-16 h-16 rounded-2xl bg-[var(--accent)] flex items-center justify-center mx-auto">
                <Terminal size={32} className="text-white" />
              </div>
              <div>
                <h2 className="text-xl font-bold text-[var(--text-primary)]">
                  Hermes Remote Manager
                </h2>
                <p className="text-sm text-[var(--text-secondary)] mt-1">
                  Version 0.1.0
                </p>
              </div>
              <p className="text-sm text-[var(--text-muted)] max-w-md mx-auto">
                A modern, fast, and secure remote connection manager built with
                Tauri and React. Manage SSH, RDP, and serial connections with
                ease.
              </p>
              <div className="flex items-center justify-center gap-6 pt-4 text-xs text-[var(--text-muted)]">
                <div>
                  <p className="font-medium text-[var(--text-secondary)]">React</p>
                  <p>18.3.1</p>
                </div>
                <div>
                  <p className="font-medium text-[var(--text-secondary)]">Tauri</p>
                  <p>2.2.0</p>
                </div>
                <div>
                  <p className="font-medium text-[var(--text-secondary)]">xterm.js</p>
                  <p>5.3.0</p>
                </div>
                <div>
                  <p className="font-medium text-[var(--text-secondary)]">Zustand</p>
                  <p>4.5.0</p>
                </div>
              </div>
              <div className="pt-4 text-xs text-[var(--text-muted)]">
                <p>© 2024 Nous Research — MIT License</p>
                <p className="mt-1">
                  Built with ❤️ for developers and sysadmins
                </p>
              </div>
            </div>
          </div>
        );
    }
  };

  return (
    <div className="h-full flex">
      {/* Tab sidebar */}
      <div className="w-48 border-r border-[var(--border)] bg-[var(--bg-secondary)] flex flex-col flex-shrink-0">
        <div className="px-4 py-3 border-b border-[var(--border)]">
          <h2 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)]">
            Settings
          </h2>
        </div>
        <nav className="flex-1 p-2 space-y-0.5 overflow-y-auto">
          {tabs.map((tab) => {
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`w-full flex items-center gap-2.5 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors text-left ${
                  isActive
                    ? 'bg-[var(--accent-muted)] text-[var(--accent)]'
                    : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
                }`}
              >
                <tab.icon size={16} className="flex-shrink-0" />
                <span>{tab.label}</span>
              </button>
            );
          })}
        </nav>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="max-w-2xl">{renderTabContent()}</div>
      </div>
    </div>
  );
}
