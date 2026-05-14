import { create } from 'zustand';
import type { Connection, Folder } from '../services/types';

interface UiStore {
  sidebarCollapsed: boolean;
  showConnectionForm: boolean;
  editingConnection: Connection | null;
  showFolderDialog: boolean;
  editingFolder: Folder | null;
  showSettings: boolean;

  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  openConnectionForm: (connection?: Connection) => void;
  closeConnectionForm: () => void;
  openFolderDialog: (folder?: Folder) => void;
  closeFolderDialog: () => void;
  setShowSettings: (show: boolean) => void;
}

export const useUiStore = create<UiStore>((set) => ({
  sidebarCollapsed: false,
  showConnectionForm: false,
  editingConnection: null,
  showFolderDialog: false,
  editingFolder: null,
  showSettings: false,

  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

  setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),

  openConnectionForm: (connection) =>
    set({ showConnectionForm: true, editingConnection: connection ?? null }),

  closeConnectionForm: () =>
    set({ showConnectionForm: false, editingConnection: null }),

  openFolderDialog: (folder) =>
    set({ showFolderDialog: true, editingFolder: folder ?? null }),

  closeFolderDialog: () =>
    set({ showFolderDialog: false, editingFolder: null }),

  setShowSettings: (show) => set({ showSettings: show }),
}));
