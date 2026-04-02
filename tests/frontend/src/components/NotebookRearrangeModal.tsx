import { useState, useEffect, useCallback } from 'react';
import { notebooksApi, notesApi, type Notebook, type Note } from '../lib/api';

interface NotebookNode {
  notebook: Notebook;
  notes: Note[];
}

interface NotebookRearrangeModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const NotebookRearrangeModal: React.FC<NotebookRearrangeModalProps> = ({
  isOpen,
  onClose,
}) => {
  const [error, setErrorLocal] = useState<string | null>(null);
  const [notebooks, setNotebooks] = useState<NotebookNode[]>([]);
  const [allNotes, setAllNotes] = useState<Note[]>([]);
  const [selectedNotebooks, setSelectedNotebooks] = useState<Set<string>>(new Set());
  const [selectedNotes, setSelectedNotes] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(false);
  const [draggedItem, setDraggedItem] = useState<{ id: string; type: 'notebook' | 'note' } | null>(null);
  const [dragOverItem, setDragOverItem] = useState<{ id: string; type: 'notebook' | 'note' } | null>(null);

  useEffect(() => {
    if (isOpen) {
      loadNotebooks();
    }
  }, [isOpen]);

  const loadNotebooks = async () => {
    try {
      setLoading(true);
      const [notebooksRes, notesRes] = await Promise.all([
        notebooksApi.getAll(),
        notesApi.getAll(),
      ]);
      
      const notebooksData = notebooksRes.data;
      const allNotesData = notesRes.data;
      setAllNotes(allNotesData);
      
      const unsortedNotes = allNotesData.filter((n: Note) => !n.notebook_id);
      
      const topLevelNotebooks = notebooksData.filter((n: Notebook) => !n.parent_id);
      
      const notebooksWithNotes = topLevelNotebooks.map((nb: Notebook) => ({
        notebook: nb,
        notes: allNotesData.filter((n: Note) => n.notebook_id === nb.id),
      }));
      
      setNotebooks([{
        notebook: {
          id: 'root',
          name: 'Unsorted',
          parent_id: null,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
        notes: unsortedNotes,
      }, ...notebooksWithNotes]);
    } catch (error) {
      console.error('Failed to load notebooks:', error);
      setErrorLocal('Failed to load notebooks for rearrangement');
    } finally {
      setLoading(false);
    }
  };

  const getNotesForNotebook = useCallback((notebookId: string | null): Note[] => {
    if (notebookId === null || notebookId === 'root') {
      return allNotes.filter(n => !n.notebook_id);
    }
    return allNotes.filter(n => n.notebook_id === notebookId);
  }, [allNotes]);

  const toggleNotebookSelection = (notebookId: string, e?: React.MouseEvent) => {
    e?.stopPropagation();
    setSelectedNotebooks(prev => {
      const newSet = new Set(prev);
      if (newSet.has(notebookId)) {
        newSet.delete(notebookId);
      } else {
        newSet.add(notebookId);
      }
      return newSet;
    });
  };

  const toggleNoteSelection = (noteId: string, e?: React.MouseEvent) => {
    e?.stopPropagation();
    setSelectedNotes(prev => {
      const newSet = new Set(prev);
      if (newSet.has(noteId)) {
        newSet.delete(noteId);
      } else {
        newSet.add(noteId);
      }
      return newSet;
    });
  };

  const handleDragStart = (e: React.DragEvent, id: string, type: 'notebook' | 'note') => {
    setDraggedItem({ id, type });
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e: React.DragEvent, id: string, type: 'notebook' | 'note') => {
    e.preventDefault();
    setDragOverItem({ id, type });
  };

  const handleDragLeave = () => {
    setDragOverItem(null);
  };

  const handleDrop = async (e: React.DragEvent, targetId: string, targetType: 'notebook' | 'note') => {
    e.preventDefault();
    setDragOverItem(null);
    
    if (!draggedItem) return;
    
    if (draggedItem.id === targetId) return;
    
    try {
      setLoading(true);
      
      const reorderedItems: Array<{
        id: string;
        type: 'notebook' | 'note';
        parent_id?: string;
        order?: number;
      }> = [];
      
      if (targetType === 'notebook' && draggedItem.type === 'notebook') {
        reorderedItems.push({
          id: draggedItem.id,
          type: 'notebook',
          parent_id: targetId === 'root' ? undefined : targetId,
        });
      } else if (targetType === 'notebook' && draggedItem.type === 'note') {
        reorderedItems.push({
          id: draggedItem.id,
          type: 'note',
          parent_id: targetId === 'root' ? undefined : targetId,
        });
      } else if (targetType === 'note' && draggedItem.type === 'note') {
        const draggedNote = allNotes.find((n: Note) => n.id === draggedItem.id);
        if (draggedNote && draggedNote.notebook_id) {
          reorderedItems.push({
            id: draggedItem.id,
            type: 'note',
            parent_id: draggedNote.notebook_id,
          });
        }
      }
      
      if (reorderedItems.length > 0) {
        try {
          if (draggedItem.type === 'notebook' || (draggedItem.type === 'note' && targetType === 'notebook')) {
            await notebooksApi.reorder({ items: reorderedItems });
          } else {
            await notesApi.reorder({ items: reorderedItems });
          }
          await loadNotebooks();
        } catch (err) {
          console.error('Reorder API error:', err);
          const errorMessage = err instanceof Error ? err.message : 'Failed to reorder items';
          setErrorLocal(`Reorder failed: ${errorMessage}`);
          throw err;
        }
      }
    } catch (error) {
      console.error('Failed to reorder:', error);
      setErrorLocal('Failed to reorder items');
    } finally {
      setDraggedItem(null);
      setLoading(false);
    }
  };

  const handleDeleteSelected = async () => {
    const notebookIds = Array.from(selectedNotebooks);
    const noteIds = Array.from(selectedNotes);
    
    if (notebookIds.length === 0 && noteIds.length === 0) {
      return;
    }
    
    if (!window.confirm(`Are you sure you want to delete ${notebookIds.length} notebook(s) and ${noteIds.length} note(s)?`)) {
      return;
    }
    
    try {
      setLoading(true);
      
      if (notebookIds.length > 0) {
        await notesApi.bulkDelete({ ids: notebookIds, type: 'notebook' });
      }
      if (noteIds.length > 0) {
        await notesApi.bulkDelete({ ids: noteIds, type: 'note' });
      }
      
      await loadNotebooks();
      setSelectedNotebooks(new Set());
      setSelectedNotes(new Set());
    } catch (error) {
      console.error('Failed to delete:', error);
      setErrorLocal('Failed to delete selected items');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      setLoading(true);
      onClose();
    } finally {
      setLoading(false);
    }
  };

  const renderNotebookNode = (node: NotebookNode) => {
    const isSelected = selectedNotebooks.has(node.notebook.id);
    const notes = getNotesForNotebook(node.notebook.id);
    
    return (
      <div key={node.notebook.id}>
        <div
          draggable
          onDragStart={(e) => handleDragStart(e, node.notebook.id, 'notebook')}
          onDragOver={(e) => handleDragOver(e, node.notebook.id, 'notebook')}
          onDragLeave={handleDragLeave}
          onDrop={(e) => handleDrop(e, node.notebook.id, 'notebook')}
          onClick={(e) => toggleNotebookSelection(node.notebook.id, e)}
          className={`
            flex items-center px-3 py-2 rounded-lg cursor-pointer transition-colors
            ${isSelected 
              ? 'bg-blue-100 dark:bg-blue-900 ring-2 ring-blue-500 dark:ring-blue-400' 
              : 'hover:bg-gray-100 dark:hover:bg-gray-700'}
            ${dragOverItem?.id === node.notebook.id && dragOverItem?.type === 'notebook' ? 'bg-blue-50 dark:bg-blue-800' : ''}
          `}
        >
          <div className="flex-1 flex items-center">
            <div className="mr-3 text-gray-500 dark:text-gray-400">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2-2z" />
              </svg>
            </div>
            <div className="flex items-center">
              <span className="text-sm font-medium text-gray-900 dark:text-white mr-2">{node.notebook.name}</span>
              <span className="text-xs text-gray-500 dark:text-gray-400">{notes.length}</span>
            </div>
          </div>
          <div className="flex items-center">
            <div className="mr-2 flex items-center">
              <svg className="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8h16M4 16h16" />
              </svg>
              <span className="text-xs text-gray-500 dark:text-gray-400 ml-1">{node.notes.length}</span>
            </div>
          </div>
        </div>
      </div>
    );
  };

  const renderNotes = (notes: Note[]) => {
    if (notes.length === 0) {
      return (
        <div className="px-4 py-8 text-center text-gray-500 dark:text-gray-400 text-sm">
          No notes in this folder
        </div>
      );
    }

    return (
      <div className="space-y-1">
        {notes.map((note) => {
          const isSelected = selectedNotes.has(note.id);
          return (
            <div
              key={note.id}
              draggable
              onDragStart={(e) => handleDragStart(e, note.id, 'note')}
              onDragOver={(e) => handleDragOver(e, note.id, 'note')}
              onDragLeave={handleDragLeave}
              onDrop={(e) => handleDrop(e, note.id, 'note')}
              onClick={(e) => toggleNoteSelection(note.id, e)}
              className={`
                flex items-center px-4 py-3 rounded-lg cursor-pointer transition-colors
                ${isSelected 
                  ? 'bg-blue-100 dark:bg-blue-900 ring-2 ring-blue-500 dark:ring-blue-400' 
                  : 'hover:bg-gray-100 dark:hover:bg-gray-700'}
                ${dragOverItem?.id === note.id && dragOverItem?.type === 'note' ? 'bg-blue-50 dark:bg-blue-800' : ''}
              `}
            >
              <div className="mr-3 text-gray-400">
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-gray-900 dark:text-white truncate">{note.title || 'Untitled'}</p>
                <p className="text-xs text-gray-500 dark:text-gray-400 truncate">{note.content.slice(0, 100)}{note.content.length > 100 ? '...' : ''}</p>
              </div>
              <span className="text-xs text-gray-500 dark:text-gray-400 ml-2 whitespace-nowrap">
                {new Date(note.updated_at).toLocaleDateString()}
              </span>
            </div>
          );
        })}
      </div>
    );
  };

  const renderNotesSection = (node: NotebookNode) => {
    const notes = getNotesForNotebook(node.notebook.id);
    return (
      <div key={node.notebook.id}>
        <div className="px-4 py-2">
          <h4 className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
            {node.notebook.name} ({notes.length})
          </h4>
          {renderNotes(notes)}
        </div>
      </div>
    );
  };

  console.log('[NotebookRearrangeModal] isOpen:', isOpen, 'returning null:', !isOpen);
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm p-4">
      <div className="w-full max-w-5xl h-[85vh] rounded-lg bg-white dark:bg-gray-800 shadow-2xl flex flex-col">
        {error && (
          <div className="p-4 bg-red-100 dark:bg-red-900 border-b border-red-300 dark:border-red-700">
            <div className="flex items-center gap-2 text-red-800 dark:text-red-200 text-sm">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              <span>{error}</span>
              <button
                onClick={() => setErrorLocal(null)}
                className="ml-auto text-red-600 dark:text-red-300 hover:underline"
              >
                Dismiss
              </button>
            </div>
          </div>
        )}
        <div className="p-6 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Rearrange Notebooks & Notes</h2>
            <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
              Drag notebooks to reorder. Drag notes to move between notebooks.
            </p>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          >
            <svg className="w-6 h-6 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="flex-1 overflow-hidden flex flex-col md:flex-row">
          <div className="w-full md:w-1/3 border-r border-gray-200 dark:border-gray-700 flex flex-col">
            <div className="p-4 border-b border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Notebooks</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                {selectedNotebooks.size} selected
              </p>
            </div>
            <div className="flex-1 overflow-y-auto p-2">
              {loading ? (
                <div className="flex items-center justify-center h-full text-gray-500">
                  Loading...
                </div>
              ) : (
                <>
                  {notebooks.map((node) => renderNotebookNode(node))}
                  {notebooks.length === 0 && (
                    <div className="flex flex-col items-center justify-center py-12 text-gray-500">
                      <p className="text-sm">No notebooks yet</p>
                    </div>
                  )}
                </>
              )}
            </div>
          </div>

          <div className="flex-1 flex flex-col">
            <div className="p-4 border-b border-gray-200 dark:border-gray-700">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Notes</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                {selectedNotes.size} selected • Drag notes to reorder or move to different notebooks
              </p>
            </div>
            <div className="flex-1 overflow-y-auto">
              {loading ? (
                <div className="flex items-center justify-center h-full text-gray-500">
                  Loading...
                </div>
              ) : (
                <>
                  {notebooks.map(node => renderNotesSection(node))}
                </>
              )}
            </div>
          </div>
        </div>

        <div className="p-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 flex flex-col md:flex-row items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2 bg-gray-200 dark:bg-gray-700 rounded-lg px-3 py-1.5">
              <span className="text-sm text-gray-700 dark:text-gray-300">
                {selectedNotebooks.size} notebooks, {selectedNotes.size} notes selected
              </span>
            </div>
            {selectedNotebooks.size > 0 || selectedNotes.size > 0 ? (
              <button
                onClick={handleDeleteSelected}
                className="flex items-center px-3 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm font-medium transition-colors"
              >
                <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
                Delete Selected
              </button>
            ) : null}
          </div>
          
          <div className="flex items-center gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded-lg text-sm font-medium transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={loading}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-lg text-sm font-medium transition-colors flex items-center"
            >
              {loading ? (
                <>
                  <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  Saving...
                </>
              ) : (
                'Save Changes'
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
