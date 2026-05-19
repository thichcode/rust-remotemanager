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
  console.log('[TerminalPage] mounted, sessionId:', sessionId, 'session:', session ? 'found' : 'NOT FOUND', 'state:', session?.state);

  // Listen to Rust-side state change events and sync to store
  useEffect(() => {
    if (!sessionId) {
      console.log('[TerminalPage] no sessionId, skipping setup');
      return;
    }

    console.log('[TerminalPage] setting up listeners for', sessionId);

    // Fallback: sync state from Rust in case events were already emitted before mount
    (async () => {
      console.log('[TerminalPage] fetching getSessionState for', sessionId);
      const result = await getSessionState(sessionId);
      console.log('[TerminalPage] getSessionState result:', JSON.stringify(result));
      if (result.success && result.data) {
        console.log('[TerminalPage] updating session state to:', result.data);
        updateSessionState(sessionId, result.data as TerminalSessionState);
      }
    })();

    let connectedUnlisten: (() => void) | undefined;
    let errorUnlisten: (() => void) | undefined;
    let exitUnlisten: (() => void) | undefined;

    const setup = async () => {
      console.log('[TerminalPage] registering event listeners...');

      connectedUnlisten = await listenToTerminalConnected(sessionId, (payload) => {
        console.log('[TerminalPage] terminal:connected received:', JSON.stringify(payload));
        if (payload.id === sessionId) {
          updateSessionState(sessionId, 'connected');
        }
      });

      errorUnlisten = await listenToTerminalError(sessionId, (payload) => {
        console.log('[TerminalPage] terminal:error received:', JSON.stringify(payload));
        if (payload.id === sessionId) {
          updateSessionState(sessionId, 'error');
        }
      });

      exitUnlisten = await listenToTerminalExit(sessionId, (payload) => {
        console.log('[TerminalPage] terminal:exit received:', JSON.stringify(payload));
        if (payload.id === sessionId) {
          updateSessionState(sessionId, 'disconnected');
        }
      });

      console.log('[TerminalPage] all listeners registered');
    };

    setup();

    return () => {
      console.log('[TerminalPage] cleanup, removing listeners for', sessionId);
      connectedUnlisten?.();
      errorUnlisten?.();
      exitUnlisten?.();
    };
  }, [sessionId, updateSessionState]);

  const handleClose = async () => {
    console.log('[TerminalPage] handleClose called, sessionId:', sessionId);
    if (sessionId) {
      await disconnectSession(sessionId);
      console.log('[TerminalPage] disconnectSession done, removing from store');
      removeSession(sessionId);
    }
    console.log('[TerminalPage] navigating to /connections');
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