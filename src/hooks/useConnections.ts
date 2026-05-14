import { useState, useEffect, useCallback } from 'react';
import type { Connection, Folder, ConnectionFormData } from '../services/types';
import {
  listConnections,
  createConnection as apiCreateConnection,
  updateConnection as apiUpdateConnection,
  deleteConnection as apiDeleteConnection,
  listFolders,
  createFolder as apiCreateFolder,
  updateFolder as apiUpdateFolder,
  deleteFolder as apiDeleteFolder,
} from '../services/ipc';
import { useConnectionStore } from '../stores/connectionStore';
import toast from 'react-hot-toast';

interface UseConnectionsReturn {
  connections: Connection[];
  folders: Folder[];
  loading: boolean;
  refreshing: boolean;
  refresh: () => Promise<void>;
  createConnection: (data: ConnectionFormData) => Promise<Connection | null>;
  updateConnection: (data: Connection) => Promise<Connection | null>;
  deleteConnection: (id: string) => Promise<boolean>;
  createFolder: (data: Omit<Folder, 'id' | 'createdAt' | 'updatedAt'>) => Promise<Folder | null>;
  updateFolder: (data: Folder) => Promise<Folder | null>;
  deleteFolder: (id: string) => Promise<boolean>;
}

export function useConnections(): UseConnectionsReturn {
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);

  const {
    connections,
    folders,
    setConnections,
    addConnection,
    updateConnection: updateConnectionInStore,
    removeConnection,
    setFolders,
    addFolder,
    updateFolder: updateFolderInStore,
    removeFolder,
  } = useConnectionStore();

  const fetchAll = useCallback(async () => {
    try {
      const [connResult, folderResult] = await Promise.all([
        listConnections(),
        listFolders(),
      ]);

      if (connResult.success && connResult.data) {
        setConnections(connResult.data);
      }
      if (folderResult.success && folderResult.data) {
        setFolders(folderResult.data);
      }
    } catch (err) {
      console.error('Failed to fetch connections/folders:', err);
    }
  }, [setConnections, setFolders]);

  // Initial fetch
  useEffect(() => {
    const init = async () => {
      setLoading(true);
      await fetchAll();
      setLoading(false);
    };
    init();
  }, [fetchAll]);

  const refresh = useCallback(async () => {
    setRefreshing(true);
    await fetchAll();
    setRefreshing(false);
  }, [fetchAll]);

  const createConnection = useCallback(
    async (data: ConnectionFormData): Promise<Connection | null> => {
      const result = await apiCreateConnection(data);
      if (result.success && result.data) {
        addConnection(result.data);
        toast.success(`Connection "${result.data.name}" created`);
        return result.data;
      }
      toast.error(result.error ?? 'Failed to create connection');
      return null;
    },
    [addConnection],
  );

  const updateConnection = useCallback(
    async (data: Connection): Promise<Connection | null> => {
      const result = await apiUpdateConnection(data);
      if (result.success && result.data) {
        updateConnectionInStore(result.data);
        toast.success(`Connection "${result.data.name}" updated`);
        return result.data;
      }
      toast.error(result.error ?? 'Failed to update connection');
      return null;
    },
    [updateConnectionInStore],
  );

  const deleteConnection = useCallback(
    async (id: string): Promise<boolean> => {
      const result = await apiDeleteConnection(id);
      if (result.success) {
        removeConnection(id);
        toast.success('Connection deleted');
        return true;
      }
      toast.error(result.error ?? 'Failed to delete connection');
      return false;
    },
    [removeConnection],
  );

  const createFolder = useCallback(
    async (
      data: Omit<Folder, 'id' | 'createdAt' | 'updatedAt'>,
    ): Promise<Folder | null> => {
      const result = await apiCreateFolder(data);
      if (result.success && result.data) {
        addFolder(result.data);
        toast.success(`Folder "${result.data.name}" created`);
        return result.data;
      }
      toast.error(result.error ?? 'Failed to create folder');
      return null;
    },
    [addFolder],
  );

  const updateFolder = useCallback(
    async (data: Folder): Promise<Folder | null> => {
      const result = await apiUpdateFolder(data);
      if (result.success && result.data) {
        updateFolderInStore(result.data);
        toast.success(`Folder "${result.data.name}" updated`);
        return result.data;
      }
      toast.error(result.error ?? 'Failed to update folder');
      return null;
    },
    [updateFolderInStore],
  );

  const deleteFolder = useCallback(
    async (id: string): Promise<boolean> => {
      const result = await apiDeleteFolder(id);
      if (result.success) {
        removeFolder(id);
        toast.success('Folder deleted');
        return true;
      }
      toast.error(result.error ?? 'Failed to delete folder');
      return false;
    },
    [removeFolder],
  );

  return {
    connections,
    folders,
    loading,
    refreshing,
    refresh,
    createConnection,
    updateConnection,
    deleteConnection,
    createFolder,
    updateFolder,
    deleteFolder,
  };
}
