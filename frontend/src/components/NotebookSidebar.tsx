import { useState, useEffect } from 'react';
import { notebooksApi, notesApi, type Notebook } from '../lib/api';
import { useNoteStore } from '../hooks/useNoteStore';
import { NotebookRearrangeModal } from './NotebookRearrangeModal';

interface NotebookSidebarProps {
  selectedNotebookId: string | null;
  onOpenNotes: (notebook: Notebook | null) => void;
  onNoteSelect?: (notebookId: string | null) => void;
  isOpen?: boolean;
}

interface NotebookSidebarProps {
  selectedNotebookId: string | null;
  onOpenNotes: (notebook: Notebook | null) => void;
  onNoteSelect?: (notebookId: string | null) => void;
  isOpen?: boolean;
}

export const NotebookSidebar = ({ 
  selectedNotebookId, 
  onOpenNotes,
  onNoteSelect
}: NotebookSidebarProps) => {
  const { notebooks, setNotebooks, setError } = useNoteStore();
  const [loading, setLoadingState] = useState(true);
  const [unsortedNotes, setUnsortedNotes] = useState<number>(0);
  const [notebookCounts, setNotebookCounts] = useState<Record<string, number>>({});
  const [isRearrangeModalOpen, setIsRearrangeModalOpen] = useState(false);

  useEffect(() => {
    const loadNotebooks = async () => {
      try {
        const [notebooksRes, notesRes] = await Promise.all([
          notebooksApi.getAll(),
          notesApi.getAll(),
        ]);
        setNotebooks(notebooksRes.data);
        const allNotes = notesRes.data;
        setUnsortedNotes(allNotes.filter(n => !n.notebook_id).length);
        
        const counts: Record<string, number> = {};
        allNotes.forEach(n => {
          if (n.notebook_id) {
            counts[n.notebook_id] = (counts[n.notebook_id] || 0) + 1;
          }
        });
        setNotebookCounts(counts);
      } catch (error) {
        console.error('Failed to fetch notebooks:', error);
        setError('Failed to load notebooks');
      } finally {
        setLoadingState(false);
      }
    };
    loadNotebooks();
  }, [setError, setNotebooks]);

  const handleNotebookClick = (notebook: Notebook | null) => {
    if (notebook) {
      onNoteSelect?.(notebook.id);
    } else {
      onNoteSelect?.(null);
    }
    onOpenNotes(notebook);
  };

  const handleCreateNotebook = async () => {
    const name = prompt('Enter notebook name:');
    if (name?.trim()) {
      try {
        await notebooksApi.create({ name: name.trim() });
        const response = await notebooksApi.getAll();
        setNotebooks(response.data);
        setLoadingState(false);
      } catch (error) {
        console.error('Failed to create notebook:', error);
        setLoadingState(false);
      }
    }
  };

  const handleOpenRearrange = () => {
    setIsRearrangeModalOpen(true);
  };

  const handleCloseRearrange = () => {
    setIsRearrangeModalOpen(false);
  };

  return (
      <div className="w-full bg-gray-50 dark:bg-gray-800 flex flex-col h-full">
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Notebooks</h2>
      </div>
      
      <div className="flex-1 overflow-y-auto p-3 space-y-2">
        {loading ? (
          <div className="flex items-center justify-center h-full text-gray-500">
            Loading...
          </div>
        ) : (
          <div className="space-y-2">
            <div
              onClick={() => handleNotebookClick(null)}
              className={`
                cursor-pointer rounded-xl transition-all duration-200
                ${selectedNotebookId === null 
                  ? 'bg-blue-100 dark:bg-blue-900 ring-2 ring-blue-500 dark:ring-blue-400 shadow-sm' 
                  : 'bg-white dark:bg-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600 ring-1 ring-gray-200 dark:ring-gray-600 shadow-sm hover:shadow-md'}
              `}
            >
              <div className="px-4 py-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-semibold text-gray-900 dark:text-white">Unsorted</span>
                  <span className="text-xs font-medium text-gray-500 dark:text-gray-400">{unsortedNotes}</span>
                </div>
              </div>
            </div>
            {notebooks.map((notebook) => (
              <div
                key={notebook.id}
                onClick={() => handleNotebookClick(notebook)}
                className={`
                  cursor-pointer rounded-xl transition-all duration-200
                  ${selectedNotebookId === notebook.id 
                    ? 'bg-blue-100 dark:bg-blue-900 ring-2 ring-blue-500 dark:ring-blue-400 shadow-sm' 
                    : 'bg-white dark:bg-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600 ring-1 ring-gray-200 dark:ring-gray-600 shadow-sm hover:shadow-md'}
                `}
              >
                <div className="px-4 py-3">
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-semibold text-gray-900 dark:text-white truncate flex-1">{notebook.name}</span>
                    <span className="text-xs font-medium text-gray-500 dark:text-gray-400">{notebookCounts[notebook.id] || 0}</span>
                  </div>
                </div>
              </div>
            ))}
            {notebooks.length === 0 && (
              <div className="flex flex-col items-center justify-center py-8 text-gray-500">
                <p className="text-sm">No notebooks yet</p>
              </div>
            )}
          </div>
        )}
      </div>

      <div className="p-2 border-t border-gray-200 dark:border-gray-700 flex gap-2">
        <button
          onClick={handleCreateNotebook}
          className="flex-1 p-1.5 bg-blue-600 hover:bg-blue-700 text-white rounded-md transition-colors flex items-center justify-center"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
        </button>
        <button
          onClick={handleOpenRearrange}
          className="flex-1 p-1.5 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-md transition-colors flex items-center justify-center"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8h16M4 16h16" />
          </svg>
        </button>
      </div>

      <NotebookRearrangeModal
        isOpen={isRearrangeModalOpen}
        onClose={handleCloseRearrange}
      />
    </div>
  );
};
