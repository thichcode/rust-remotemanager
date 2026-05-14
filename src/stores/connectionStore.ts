import { create } from 'zustand';
import type { Connection, Folder, ConnectionType } from '../services/types';

interface ConnectionStore {
  connections: Connection[];
  folders: Folder[];
  selectedConnectionId: string | null;
  searchTerm: string;
  filterType: ConnectionType | 'all';

  setConnections: (connections: Connection[]) => void;
  addConnection: (connection: Connection) => void;
  updateConnection: (connection: Connection) => void;
  removeConnection: (id: string) => void;
  setFolders: (folders: Folder[]) => void;
  addFolder: (folder: Folder) => void;
  updateFolder: (folder: Folder) => void;
  removeFolder: (id: string) => void;
  setSelectedConnectionId: (id: string | null) => void;
  setSearchTerm: (term: string) => void;
  setFilterType: (type: ConnectionType | 'all') => void;
}

export const useConnectionStore = create<ConnectionStore>((set) => ({
  connections: [],
  folders: [],
  selectedConnectionId: null,
  searchTerm: '',
  filterType: 'all',

  setConnections: (connections) => set({ connections }),

  addConnection: (connection) =>
    set((state) => ({ connections: [...state.connections, connection] })),

  updateConnection: (connection) =>
    set((state) => ({
      connections: state.connections.map((c) =>
        c.id === connection.id ? connection : c,
      ),
    })),

  removeConnection: (id) =>
    set((state) => ({
      connections: state.connections.filter((c) => c.id !== id),
      selectedConnectionId:
        state.selectedConnectionId === id ? null : state.selectedConnectionId,
    })),

  setFolders: (folders) => set({ folders }),

  addFolder: (folder) =>
    set((state) => ({ folders: [...state.folders, folder] })),

  updateFolder: (folder) =>
    set((state) => ({
      folders: state.folders.map((f) => (f.id === folder.id ? folder : f)),
    })),

  removeFolder: (id) =>
    set((state) => ({
      folders: state.folders.filter((f) => f.id !== id),
    })),

  setSelectedConnectionId: (id) => set({ selectedConnectionId: id }),
  setSearchTerm: (term) => set({ searchTerm: term }),
  setFilterType: (type) => set({ filterType: type }),
}));
