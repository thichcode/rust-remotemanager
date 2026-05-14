import { create } from 'zustand';
import type { TerminalSession, TerminalSessionState } from '../services/types';

interface SessionStore {
  sessions: TerminalSession[];
  activeSessionId: string | null;

  addSession: (session: TerminalSession) => void;
  removeSession: (id: string) => void;
  setActiveSession: (id: string | null) => void;
  updateSessionState: (id: string, state: TerminalSessionState) => void;
  clearSessions: () => void;
}

export const useSessionStore = create<SessionStore>((set) => ({
  sessions: [],
  activeSessionId: null,

  addSession: (session) =>
    set((state) => ({
      sessions: [...state.sessions, session],
      activeSessionId: session.id,
    })),

  removeSession: (id) =>
    set((state) => {
      const filtered = state.sessions.filter((s) => s.id !== id);
      const wasActive = state.activeSessionId === id;
      return {
        sessions: filtered,
        activeSessionId: wasActive
          ? filtered.length > 0
            ? filtered[filtered.length - 1].id
            : null
          : state.activeSessionId,
      };
    }),

  setActiveSession: (id) => set({ activeSessionId: id }),

  updateSessionState: (id, state) =>
    set((prev) => ({
      sessions: prev.sessions.map((s) =>
        s.id === id ? { ...s, state } : s,
      ),
    })),

  clearSessions: () => set({ sessions: [], activeSessionId: null }),
}));
