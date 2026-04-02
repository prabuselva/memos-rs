import { useState, useEffect } from 'react';
import { notesApi, type Note, type Notebook } from '../lib/api';
import { useNoteStore } from '../hooks/useNoteStore';
import { NoteCard } from './NoteCard';

interface NotesSidebarProps {
  notebook: Notebook | null;
  onClose: () => void;
  onSelectNote: (noteId: string) => void;
  onNoteSelect?: (noteId: string) => void;
}

export const NotesSidebar = ({ notebook, onClose, onSelectNote, onNoteSelect }: NotesSidebarProps) => {
  const { notes, setError } = useNoteStore();
  const [loading, setLoading] = useState(true);
  const [filteredNotes, setFilteredNotes] = useState<Note[]>([]);
  const [show, setShow] = useState(false);

  useEffect(() => {
    const loadNotes = async () => {
      if (notebook === null) {
        const rootNotes = notes.filter(n => !n.notebook_id);
        setFilteredNotes(rootNotes);
      } else {
        try {
          const response = await notesApi.getAll();
          const notebookNotes = response.data.filter(n => n.notebook_id === notebook.id);
          setFilteredNotes(notebookNotes);
        } catch (error) {
          console.error('Failed to fetch notes:', error);
          setError('Failed to load notes');
        }
      }
      setLoading(false);
      setShow(true);
    };
    loadNotes();
  }, [notebook, notes, setError]);

  const handleSelectNote = (noteId: string) => {
    onNoteSelect?.(noteId);
    onSelectNote(noteId);
  };

  const handleClose = () => {
    setShow(false);
    setTimeout(() => {
      onClose();
    }, 300);
  };

  return (
    <div
      className={`
        absolute inset-0 bg-white dark:bg-gray-800
        transition-transform duration-300 ease-in-out
        ${show ? 'translate-x-0' : 'translate-x-full'}
      `}
    >
      <div className="h-full flex flex-col">
        <div className="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
          <button
            onClick={handleClose}
            className="flex items-center text-gray-700 dark:text-gray-200 hover:text-blue-600 dark:hover:text-blue-400 transition-colors px-3 py-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700"
          >
            <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
            </svg>
            Back
          </button>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            {notebook === null ? 'Unsorted Notes' : notebook.name}
          </h2>
        </div>
        
        <div className="flex-1 overflow-y-auto p-4">
          {loading ? (
            <div className="flex items-center justify-center h-full text-gray-500">
              Loading...
            </div>
          ) : filteredNotes.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-gray-500 p-4">
              <p className="text-sm">No notes in this folder</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 gap-3">
              {filteredNotes.map((note) => (
                <NoteCard
                  key={note.id}
                  note={note}
                  onClick={() => handleSelectNote(note.id)}
                  isSelected={false}
                />
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
