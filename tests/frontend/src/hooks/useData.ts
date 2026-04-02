import { useEffect } from 'react';
import { useNoteStore } from '../hooks/useNoteStore';
import { notesApi, notebooksApi } from '../lib/api';
import { useAuthStore } from '../hooks/useAuthStore';

const detectDarkMode = () => {
  if (typeof window !== 'undefined' && window.matchMedia) {
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  }
  return false;
};

export const useInitializeData = () => {
  const { setNotes, setNotebooks, setDarkMode, setLoading, setError } = useNoteStore();
  const { isAuthenticated, isLoading: authLoading } = useAuthStore();

  useEffect(() => {
    if (!isAuthenticated || authLoading) {
      return;
    }

    const loadData = async () => {
      setLoading(true);
      try {
        const [notesRes, notebooksRes] = await Promise.all([
          notesApi.getAll(),
          notebooksApi.getAll(),
        ]);

        setNotes(notesRes.data);
        setNotebooks(notebooksRes.data);
        setDarkMode(detectDarkMode());
      } catch {
        setError('Failed to load data. Please check if the server is running.');
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, [isAuthenticated, authLoading, setNotes, setNotebooks, setDarkMode, setLoading, setError]);
};