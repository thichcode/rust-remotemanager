import { useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useSessionStore } from '../stores/sessionStore';
import {
  disconnectSession,
  listenToTerminalConnected,
  listenToTerminalError,
  listenToTerminalExit,
  getSessionState,
} from '../services/ipc';
import TerminalSessionComponent from '../components/terminal/TerminalSession';
import type { TerminalSessionState } from '../services/types';

export default function TerminalPage() {
  const { sessionId } = useParams<{ sessionId: string }>();
  const navigate = useNavigate();
  const { sessions, updateSessionState, removeSession } = useSessionStore();

  const session = sessions.find((s) => s.id === sessionId);

  // Listen to Rust-side state change events and sync to store
  useEffect(() => {
    if (!sessionId) return;

    // Fallback: sync state from Rust in case events were already emitted before mount
    (async () => {
      const result = await getSessionState(sessionId);
      if (result.success && result.data) {
        updateSessionState(sessionId, result.data as TerminalSessionState);
      }
    })();

    let connectedUnlisten: (() => void) | undefined;
    let errorUnlisten: (() => void) | undefined;
    let exitUnlisten: (() => void) | undefined;

    const setup = async () => {
      connectedUnlisten = await listenToTerminalConnected(sessionId, (payload) => {
        if (payload.id === sessionId) {
          updateSessionState(sessionId, 'connected');
        }
      });

      errorUnlisten = await listenToTerminalError(sessionId, (payload) => {
        if (payload.id === sessionId) {
          updateSessionState(sessionId, 'error');
        }
      });

      exitUnlisten = await listenToTerminalExit(sessionId, (payload) => {
        if (payload.id === sessionId) {
          updateSessionState(sessionId, 'disconnected');
        }
      });
    };

    setup();

    return () => {
      connectedUnlisten?.();
      errorUnlisten?.();
      exitUnlisten?.();
    };
  }, [sessionId, updateSessionState]);

  const handleClose = async () => {
    if (sessionId) {
      await disconnectSession(sessionId);
      removeSession(sessionId);
    }
    navigate('/connections');
  };

  if (!session) {
    return (
      <div className="flex items-center justify-center h-full bg-[var(--bg-primary)]">
        <div className="text-center">
          <p className="text-sm text-[var(--text-secondary)]">Session not found</p>
          <button
            onClick={() => navigate('/connections')}
            className="mt-4 inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--accent)] text-white text-sm font-medium hover:bg-[var(--accent-hover)] transition-colors"
          >
            Back to connections
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full w-full">
      <TerminalSessionComponent session={session} onClose={handleClose} />
    </div>
  );
}