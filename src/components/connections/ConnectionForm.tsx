import { useState, useEffect } from 'react';
import {
  X,
  Save,
  Terminal,
  Monitor,
  Cable,
  Key,
  FolderOpen,
  Hash,
  Tags,
  FileText,
  Heart,
  FileKey,
  Lock,
} from 'lucide-react';
import { Connection, ConnectionFormData, ConnectionType, AuthType, Folder } from '../../services/types';
import { useConnectionStore } from '../../stores/connectionStore';
import { getCredentials, pickSSHKeyFile, saveCredential } from '../../services/ipc';
import type { Credential } from '../../services/types';

interface ConnectionFormProps {
  editingConnection?: Connection | null;
  onSave: (data: ConnectionFormData) => void;
  onCancel: () => void;
}

const initialFormData: ConnectionFormData = {
  name: '',
  type: ConnectionType.SSH,
  host: '',
  port: 22,
  username: '',
  authType: 'password' as AuthType,
  tags: [],
  notes: '',
  startupCommands: '',
  keepaliveInterval: 0,
  isFavorite: false,
};

export default function ConnectionForm({
  editingConnection,
  onSave,
  onCancel,
}: ConnectionFormProps) {
  const [formData, setFormData] = useState<ConnectionFormData>(initialFormData);
  const [credentials, setCredentials] = useState<Credential[]>([]);
  const [tagInput, setTagInput] = useState('');
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [keyFilePath, setKeyFilePath] = useState<string>('');
  const [keyName, setKeyName] = useState<string>('');
  const [savingKey, setSavingKey] = useState(false);
  const { folders } = useConnectionStore();

  useEffect(() => {
    if (editingConnection) {
      setFormData({
        name: editingConnection.name,
        type: editingConnection.type,
        folderId: editingConnection.folderId,
        host: editingConnection.host,
        port: editingConnection.port,
        username: editingConnection.username,
        credentialId: editingConnection.credentialId,
        authType: editingConnection.authType,
        proxyType: editingConnection.proxyType,
        proxyHost: editingConnection.proxyHost,
        proxyPort: editingConnection.proxyPort,
        proxyUsername: editingConnection.proxyUsername,
        tags: editingConnection.tags ?? [],
        notes: editingConnection.notes ?? '',
        startupCommands: (editingConnection.startupCommands ?? []).join('\n'),
        keepaliveInterval: editingConnection.keepaliveInterval ?? 0,
        isFavorite: editingConnection.isFavorite,
        color: editingConnection.color,
      });
    } else {
      setFormData(initialFormData);
    }
  }, [editingConnection]);

  useEffect(() => {
    const load = async () => {
      const result = await getCredentials();
      if (result.success && result.data) {
        setCredentials(result.data);
      }
    };
    load();
  }, []);

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};
    if (!formData.name.trim()) newErrors.name = 'Name is required';
    if (!formData.host.trim()) newErrors.host = 'Host is required';
    if (!formData.port || formData.port < 1 || formData.port > 65535)
      newErrors.port = 'Valid port (1-65535) required';
    if (!formData.username.trim()) newErrors.username = 'Username is required';
    if (formData.authType === 'key' && !formData.credentialId && !keyFilePath) {
      newErrors.auth = 'SSH key file is required (pick a file or select from saved credentials)';
    }
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!validate()) return;
    onSave(formData);
  };

  const updateField = <K extends keyof ConnectionFormData>(
    key: K,
    value: ConnectionFormData[K],
  ) => {
    setFormData((prev) => ({ ...prev, [key]: value }));
    if (errors[key]) {
      setErrors((prev) => {
        const { [key]: _, ...rest } = prev;
        return rest;
      });
    }
  };

  const handlePickKeyFile = async () => {
    const result = await pickSSHKeyFile();
    if (result.success && result.data) {
      setKeyFilePath(result.data);
      // Auto-generate a credential name from the filename
      const fileName = result.data.split('/').pop() || 'SSH Key';
      setKeyName(fileName);
      // Auto-save the credential
      setSavingKey(true);
      try {
        const credentialResult = await saveCredential({
          name: fileName,
          authType: 'key',
          username: formData.username || undefined,
          keyPath: result.data,
        });
        if (credentialResult.success && credentialResult.data) {
          setCredentials((prev) => [...prev, credentialResult.data]);
          updateField('credentialId', credentialResult.data.id);
          setErrors((prev) => {
            const { auth, ...rest } = prev;
            return rest;
          });
        }
      } finally {
        setSavingKey(false);
      }
    }
  };

  const handleCredentialChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const val = e.target.value;
    if (val === '__new__') {
      handlePickKeyFile();
    } else {
      updateField('credentialId', val || undefined);
    }
  };

  const addTag = () => {
    const tag = tagInput.trim().toLowerCase();
    if (tag && !formData.tags.includes(tag)) {
      updateField('tags', [...formData.tags, tag]);
    }
    setTagInput('');
  };

  const removeTag = (tag: string) => {
    updateField(
      'tags',
      formData.tags.filter((t) => t !== tag),
    );
  };

  const connectionTypes: { value: ConnectionType; label: string; icon: typeof Terminal; defaultPort: number }[] = [
    { value: ConnectionType.SSH, label: 'SSH', icon: Terminal, defaultPort: 22 },
    { value: ConnectionType.RDP, label: 'RDP', icon: Monitor, defaultPort: 3389 },
    { value: ConnectionType.Serial, label: 'Serial', icon: Cable, defaultPort: 9600 },
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-2xl max-h-[90vh] overflow-y-auto rounded-2xl border border-[var(--border)] bg-[var(--bg-secondary)] shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-[var(--border)]">
          <h2 className="text-lg font-bold text-[var(--text-primary)]">
            {editingConnection ? 'Edit Connection' : 'New Connection'}
          </h2>
          <button
            onClick={onCancel}
            className="p-1.5 rounded-md text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6 space-y-5">
          {/* Basic Info */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="sm:col-span-2">
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                Connection Name *
              </label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => updateField('name', e.target.value)}
                placeholder="My Server"
                className={`w-full rounded-lg border ${\n                  errors.name ? 'border-[var(--status-error)]' : 'border-[var(--border)]'\n                } bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors`}
              />
              {errors.name && (
                <p className="text-xs text-[var(--status-error)] mt-1">{errors.name}</p>
              )}
            </div>

            {/* Type */}
            <div>
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                Connection Type *
              </label>
              <div className="flex gap-2">
                {connectionTypes.map((t) => (
                  <button
                    key={t.value}
                    type="button"
                    onClick={() => {
                      updateField('type', t.value);
                      if (formData.port === 22 || formData.port === 3389 || formData.port === 9600) {
                        updateField('port', t.defaultPort);
                      }
                    }}
                    className={`flex items-center gap-2 px-4 py-2 rounded-lg border text-sm font-medium transition-colors ${\n                      formData.type === t.value\n                        ? 'border-[var(--accent)] bg-[var(--accent-muted)] text-[var(--accent)]'\n                        : 'border-[var(--border)] text-[var(--text-secondary)] hover:border-[var(--border-light)] hover:text-[var(--text-primary)]'\n                    }`}\n                  >
                    <t.icon size={16} />
                    {t.label}
                  </button>
                ))}
              </div>
            </div>

            {/* Folder */}
            <div>
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                Folder
              </label>
              <div className="relative">
                <FolderOpen
                  size={16}
                  className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
                />
                <select
                  value={formData.folderId ?? ''}
                  onChange={(e) =>
                    updateField('folderId', e.target.value || undefined)
                  }
                  className="w-full pl-9 pr-3 py-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors appearance-none"
                >
                  <option value="">No folder (root)</option>
                  {folders.map((f) => (
                    <option key={f.id} value={f.id}>
                      {f.name}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            {/* Host */}
            <div>
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                Hostname / IP *
              </label>
              <input
                type="text"
                value={formData.host}
                onChange={(e) => updateField('host', e.target.value)}
                placeholder="192.168.1.1"
                className={`w-full rounded-lg border ${\n                  errors.host ? 'border-[var(--status-error)]' : 'border-[var(--border)]'\n                } bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors`}
              />
              {errors.host && (
                <p className="text-xs text-[var(--status-error)] mt-1">{errors.host}</p>
              )}
            </div>

            {/* Port */}
            <div>
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                Port *
              </label>
              <div className="relative">
                <Hash
                  size={16}
                  className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
                />
                <input
                  type="number"
                  value={formData.port}
                  onChange={(e) => updateField('port', parseInt(e.target.value) || 0)}
                  placeholder="22"
                  min={1}
                  max={65535}
                  className={`w-full pl-9 pr-3 py-2 rounded-lg border ${\n                    errors.port ? 'border-[var(--status-error)]' : 'border-[var(--border)]'\n                  } bg-[var(--bg-primary)] text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors`}
                />
              </div>
              {errors.port && (
                <p className="text-xs text-[var(--status-error)] mt-1">{errors.port}</p>
              )}
            </div>

            {/* Username */}
            <div>
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                Username *
              </label>
              <input
                type="text"
                value={formData.username}
                onChange={(e) => updateField('username', e.target.value)}
                placeholder="root"
                className={`w-full rounded-lg border ${\n                  errors.username ? 'border-[var(--status-error)]' : 'border-[var(--border)]'\n                } bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors`}
              />
              {errors.username && (
                <p className="text-xs text-[var(--status-error)] mt-1">{errors.username}</p>
              )}
            </div>

            {/* Auth Type */}
            <div>
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                Authentication
              </label>
              <div className="relative">
                <Key
                  size={16}
                  className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
                />
                <select
                  value={formData.authType}
                  onChange={(e) => {
                    const val = e.target.value as AuthType;
                    updateField('authType', val);
                    if (val !== 'key') {
                      updateField('credentialId', undefined);
                    }
                  }}
                  className="w-full pl-9 pr-3 py-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors appearance-none"
                >
                  <option value="password">Password</option>
                  <option value="key">SSH Key</option>
                  <option value="agent">SSH Agent</option>
                </select>
              </div>
            </div>
          </div>

          {/* SSH Key Picker — shown when authType is 'key' */}
          {formData.authType === 'key' && (
            <div className="space-y-3 p-4 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border)]">
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                SSH Key Credential
              </label>
              <div className="flex gap-2">
                <div className="relative flex-1">
                  <Lock
                    size={16}
                    className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
                  />
                  <select
                    value={formData.credentialId || ''}
                    onChange={handleCredentialChange}
                    className="w-full pl-9 pr-3 py-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors appearance-none"
                  >
                    <option value="">— Select saved key or pick new —</option>
                    {credentials
                      .filter((c) => c.authType === 'key')
                      .map((c) => (
                        <option key={c.id} value={c.id}>
                          {c.name}
                          {c.keyPath && ` (${c.keyPath.split('/').pop()})`}
                        </option>
                      ))}
                    <option value="__new__">⚡ Pick key file from disk…</option>
                  </select>
                </div>
                <button
                  type="button"
                  onClick={handlePickKeyFile}
                  disabled={savingKey}
                  className="inline-flex items-center gap-2 px-4 py-2 rounded-lg border border-[var(--border)] text-sm font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-primary)] transition-colors disabled:opacity-50"
                >
                  <FileKey size={16} />
                  {savingKey ? 'Saving…' : 'Pick File'}
                </button>
              </div>
              {keyFilePath && (
                <p className="text-xs text-[var(--text-muted)] truncate">
                  📁 {keyFilePath}
                </p>
              )}
              {errors.auth && (
                <p className="text-xs text-[var(--status-error)]">{errors.auth}</p>
              )}
            </div>
          )}

          {/* Tags */}
          <div>
            <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
              Tags
            </label>
            <div className="flex items-center gap-2 flex-wrap mb-2">
              {formData.tags.map((tag) => (
                <span
                  key={tag}
                  className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-xs font-medium bg-[var(--bg-tertiary)] text-[var(--text-secondary)]"
                >
                  {tag}
                  <button
                    type="button"
                    onClick={() => removeTag(tag)}
                    className="hover:text-[var(--status-error)]"
                  >
                    <X size={12} />
                  </button>
                </span>
              ))}
            </div>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <Tags
                  size={16}
                  className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]"
                />
                <input
                  type="text"
                  value={tagInput}
                  onChange={(e) => setTagInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      e.preventDefault();
                      addTag();
                    }
                  }}
                  placeholder="Type a tag and press Enter..."
                  className="w-full pl-9 pr-3 py-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors"
                />
              </div>
              <button
                type="button"
                onClick={addTag}
                className="px-3 py-2 rounded-lg border border-[var(--border)] text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
              >
                Add
              </button>
            </div>
          </div>

          {/* Notes */}
          <div>
            <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
              Notes
            </label>
            <div className="relative">
              <FileText
                size={16}
                className="absolute left-3 top-3 text-[var(--text-muted)]"
              />
              <textarea
                value={formData.notes}
                onChange={(e) => updateField('notes', e.target.value)}
                placeholder="Optional description or notes..."
                rows={3}
                className="w-full pl-9 pr-3 py-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors resize-none"
              />
            </div>
          </div>

          {/* Keepalive + Favorite */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-1.5">
                Keepalive Interval (seconds)
              </label>
              <input
                type="number"
                value={formData.keepaliveInterval}
                onChange={(e) =>
                  updateField('keepaliveInterval', parseInt(e.target.value) || 0)
                }
                placeholder="0 = disabled"
                min={0}
                className="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] focus:ring-1 focus:ring-[var(--accent)] transition-colors"
              />
            </div>
            <div className="flex items-end pb-2">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={formData.isFavorite}
                  onChange={(e) => updateField('isFavorite', e.target.checked)}
                  className="w-4 h-4 rounded border-[var(--border)] bg-[var(--bg-primary)] text-[var(--accent)] focus:ring-[var(--accent)] focus:ring-offset-0"
                />
                <Heart
                  size={16}
                  className={
                    formData.isFavorite
                      ? 'text-[var(--warning)] fill-[var(--warning)]'
                      : 'text-[var(--text-muted)]'
                  }
                />
                <span className="text-sm text-[var(--text-secondary)]">
                  Mark as favorite
                </span>
              </label>
            </div>
          </div>

          {/* Actions */}
          <div className="flex items-center justify-end gap-3 pt-4 border-t border-[var(--border)]">
            <button
              type="button"
              onClick={onCancel}
              className="px-4 py-2 rounded-lg border border-[var(--border)] text-sm font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="inline-flex items-center gap-2 px-5 py-2 rounded-lg bg-[var(--accent)] text-white text-sm font-medium hover:bg-[var(--accent-hover)] active:scale-[0.98] transition-all"
            >
              <Save size={16} />
              {editingConnection ? 'Update Connection' : 'Create Connection'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}