import { useEffect, useRef, useState, useCallback } from 'react';
import { Search, ChevronUp, ChevronDown, X } from 'lucide-react';
import { useTerminal } from '../../hooks/useTerminal';
import type { TerminalSession } from '../../services/types';

interface TerminalSessionProps {
  session: TerminalSession;
  onClose?: () => void;
}

export default function TerminalSessionComponent({
  session,
  onClose,
}: TerminalSessionProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const {
    terminalRef,
    isReady,
    searchVisible,
    searchText,
    setSearchText,
    closeSearch,
    findNext,
    findPrevious,
  } = useTerminal({
    sessionId: session.id,
  });

  // Handle Ctrl+F keyboard shortcut within the terminal
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
        // Already handled in useTerminal
      }
    };

    container.addEventListener('keydown', handleKeyDown);
    return () => container.removeEventListener('keydown', handleKeyDown);
  }, []);

  // Status overlay when connecting
  if (session.state === 'connecting') {
    return (
      <div className="flex items-center justify-center h-full bg-[var(--terminal-bg)]">
        <div className="text-center space-y-3">
          <div className="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin mx-auto" />
          <p className="text-sm text-[var(--text-secondary)]">
            Connecting to session...
          </p>
        </div>
      </div>
    );
  }

  if (session.state === 'error') {
    return (
      <div className="flex items-center justify-center h-full bg-[var(--terminal-bg)]">
        <div className="text-center space-y-3 max-w-md px-6">
          <div className="w-12 h-12 rounded-full bg-[var(--status-error)]/10 flex items-center justify-center mx-auto">
            <X size={24} className="text-[var(--status-error)]" />
          </div>
          <p className="text-sm font-medium text-[var(--status-error)]">
            Connection failed
          </p>
          <p className="text-xs text-[var(--text-secondary)]">
            An error occurred while connecting to the remote host. Please check
            your connection settings and try again.
          </p>
          {onClose && (
            <button
              onClick={onClose}
              className="mt-3 px-4 py-2 rounded-lg border border-[var(--border)] text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
            >
              Close
            </button>
          )}
        </div>
      </div>
    );
  }

  return (
    <div ref={containerRef} className="relative h-full w-full bg-[#0d1117]">
      {/* Terminal container */}
      <div ref={terminalRef as React.RefObject<HTMLDivElement>} className="h-full w-full" />

      {/* Search overlay */}
      {searchVisible && (
        <div className="absolute top-3 right-3 flex items-center gap-1 bg-[var(--bg-secondary)] border border-[var(--border)] rounded-lg shadow-xl px-2 py-1.5 z-10">
          <Search size={14} className="text-[var(--text-muted)]" />
          <input
            type="text"
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.shiftKey ? findPrevious() : findNext();
              }
              if (e.key === 'Escape') {
                closeSearch();
              }
            }}
            placeholder="Search terminal..."
            autoFocus
            className="w-40 bg-transparent text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)] border-none outline-none"
          />
          <div className="flex items-center gap-0.5 border-l border-[var(--border)] pl-1.5 ml-1">
            <button
              onClick={findPrevious}
              className="p-0.5 rounded text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]"
              title="Previous match (Shift+Enter)"
            >
              <ChevronUp size={14} />
            </button>
            <button
              onClick={findNext}
              className="p-0.5 rounded text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]"
              title="Next match (Enter)"
            >
              <ChevronDown size={14} />
            </button>
            <button
              onClick={closeSearch}
              className="p-0.5 rounded text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] ml-0.5"
            >
              <X size={14} />
            </button>
          </div>
        </div>
      )}

      {/* Status indicator */}
      {session.state === 'disconnected' && (
        <div className="absolute bottom-3 left-3 flex items-center gap-1.5 px-2 py-1 rounded bg-black/60 text-xs text-[var(--text-muted)]">
          <span className="w-1.5 h-1.5 rounded-full bg-[var(--text-muted)]" />
          Disconnected
        </div>
      )}
    </div>
  );
}
