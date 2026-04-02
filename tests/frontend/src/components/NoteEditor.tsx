import { useState, useEffect } from 'react';
import { useNoteStore } from '../hooks/useNoteStore';
import { notesApi, type CreateNoteRequest } from '../lib/api';
import { MarkdownPreview } from './MarkdownPreview';

const DateFooter = ({ createdAt, updatedAt }: { createdAt?: string; updatedAt?: string }) => {
  const formatDate = (dateString?: string) => {
    if (!dateString) return '';
    return new Date(dateString).toLocaleDateString('en-US', { 
      year: 'numeric', 
      month: 'long', 
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  return (
    <div className="sticky bottom-0 left-0 right-0 bg-white dark:bg-gray-900 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-500 dark:text-gray-400 text-center py-2 z-30">
      {createdAt && <span>Created: {formatDate(createdAt)}</span>}
      {createdAt && updatedAt && createdAt !== updatedAt && <span className="mx-2">•</span>}
      {updatedAt && <span>Updated: {formatDate(updatedAt)}</span>}
    </div>
  );
};

export const NoteEditor = () => {
  const { selectedNoteId, notes, updateNote, deleteNote, selectNote, setError, setLoading, selectedNotebook, notebooks } = useNoteStore();
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [tags, setTags] = useState<string[]>([]);
  const [tagInputValue, setTagInputValue] = useState('');
  const [isEditing, setIsEditing] = useState(false);
  const [isPreviewMode, setIsPreviewMode] = useState(false);
  const [notebookId, setNotebookId] = useState<string | undefined>(undefined);

  const selectedNote = notes.find((n) => n.id === selectedNoteId);

  useEffect(() => {
    setNotebookId(selectedNotebook || undefined);
  }, [selectedNotebook]);

  useEffect(() => {
    if (selectedNote) {
      setTitle(selectedNote.title);
      setContent(selectedNote.content || '');
      setTags(selectedNote.tags || []);
      setNotebookId(selectedNote.notebook_id || selectedNotebook || undefined);
      setIsPreviewMode(true);
      setIsEditing(false);
    } else {
      setTitle('');
      setContent('');
      setTags([]);
      setNotebookId(selectedNotebook || undefined);
      setIsPreviewMode(false);
      setIsEditing(false);
    }
  }, [selectedNote, selectedNotebook]);

  const handleUpdate = async () => {
    if (!selectedNoteId) return;

    setLoading(true);
    try {
      const noteData: CreateNoteRequest = {
        title,
        content,
        notebook_id: notebookId,
        tags: tags.length > 0 ? tags : undefined,
      };

      await notesApi.update(selectedNoteId, noteData);
      updateNote({ ...selectedNote!, ...noteData });
      setError(null);
    } catch (error) {
      setError('Failed to update note');
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    if (!selectedNoteId) return;

    try {
      await notesApi.delete(selectedNoteId);
      deleteNote(selectedNoteId);
      selectNote(null);
      setError(null);
    } catch (error) {
      setError('Failed to delete note');
    }
  };

  const handleEdit = () => {
    if (selectedNote) {
      setTitle(selectedNote.title);
      setContent(selectedNote.content || '');
      setTags(selectedNote.tags || []);
      setIsEditing(true);
      setIsPreviewMode(false);
    }
  };

  const handleCancel = () => {
    if (selectedNote) {
      setTitle(selectedNote.title);
      setContent(selectedNote.content || '');
      setTags(selectedNote.tags || []);
      setIsEditing(false);
      setIsPreviewMode(true);
    }
  };

  if (!selectedNoteId && !isEditing) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-gray-500">
        <p className="text-xl">Select a note to view</p>
        <p className="text-sm mt-2">or create a new one</p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col pt-4 px-4 sm:px-6 md:px-8">
      <div className="mb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        {isEditing || isPreviewMode ? (
          <input
            type="text"
            placeholder="Note title"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            disabled={!isEditing}
            className="w-full text-2xl font-bold border-none bg-transparent focus:ring-0 placeholder-gray-400 dark:text-white disabled:opacity-70"
          />
        ) : (
          <div className="w-full text-2xl font-bold text-gray-900 dark:text-white">
            {title || 'Untitled'}
          </div>
        )}
        <div className="flex items-center gap-2 flex-wrap">
          <select
            value={notebookId || ''}
            onChange={(e) => setNotebookId(e.target.value || undefined)}
            className="text-sm rounded-md border border-gray-300 bg-gray-50 px-2 py-1 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
          >
            <option value="">Unsorted</option>
            {notebooks.map((nb) => (
              <option key={nb.id} value={nb.id}>
                {nb.name}
              </option>
            ))}
          </select>
        </div>
        <div className="flex items-center gap-2 flex-wrap">
          {isPreviewMode ? (
            <button
              onClick={handleEdit}
              className="rounded-lg bg-yellow-600 px-3 py-1.5 text-sm text-white hover:bg-yellow-700"
            >
              <span className="hidden sm:inline">Edit</span>
              <span className="sm:hidden">✎</span>
            </button>
          ) : (
            <>
              <button
                onClick={handleUpdate}
                className="rounded-lg bg-green-600 px-3 py-1.5 text-sm text-white hover:bg-green-700"
              >
                <span className="hidden sm:inline">Update</span>
                <span className="sm:hidden">✓</span>
              </button>
              <button
                onClick={handleCancel}
                className="rounded-lg bg-gray-600 px-3 py-1.5 text-sm text-white hover:bg-gray-700"
              >
                <span className="hidden sm:inline">Cancel</span>
                <span className="sm:hidden">✗</span>
              </button>
              <button
                onClick={handleDelete}
                className="rounded-lg bg-red-600 px-3 py-1.5 text-sm text-white hover:bg-red-700"
              >
                <span className="hidden sm:inline">Delete</span>
                <span className="sm:hidden">🗑</span>
              </button>
            </>
          )}
        </div>
      </div>
      {isEditing ? (
        <>
          <div className="mb-4 flex flex-wrap gap-2">
            {tags.map((tag, idx) => (
              <span key={idx} className="rounded-full bg-gray-100 px-3 py-1 dark:bg-gray-700">
                {tag}
                <button
                  onClick={() => setTags(tags.filter((_, i) => i !== idx))}
                  className="ml-2 text-gray-500 hover:text-gray-700"
                >
                  ×
                </button>
              </span>
            ))}
            <input
              type="text"
              placeholder="Add tag..."
              value={tagInputValue}
              onChange={(e) => setTagInputValue(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault();
                  const value = (e.target as HTMLInputElement).value.trim();
                  if (value && !tags.includes(value)) {
                    setTags((prevTags) => [...prevTags, value]);
                  }
                  setTagInputValue('');
                }
              }}
              className="rounded-lg border border-gray-300 bg-transparent px-3 py-1 dark:border-gray-600 dark:text-white"
            />
          </div>
          <div className="flex-1 overflow-y-auto min-h-0 w-full">
            <textarea
              placeholder="Write in Markdown..."
              value={content}
              onChange={(e) => setContent(e.target.value)}
              className="w-full h-48 sm:h-64 md:h-80 lg:h-96 rounded-lg border border-gray-300 bg-white p-3 md:p-4 dark:border-gray-600 dark:bg-gray-800 dark:text-white"
            />
            <div className="mt-4 border-t pt-4 dark:border-gray-700">
              <h3 className="mb-2 text-sm font-semibold text-gray-500">Preview</h3>
           <div className="w-full max-w-full overflow-x-auto">
               <MarkdownPreview content={content} />
             </div>
             <DateFooter createdAt={selectedNote?.created_at} updatedAt={selectedNote?.updated_at} />
             </div>
          </div>
        </>
      ) : (
          <div className="flex-1 overflow-y-auto min-h-0 w-full pb-8">
            <div className="w-full max-w-full overflow-x-auto">
              <MarkdownPreview content={content || ''} />
            </div>
            <DateFooter createdAt={selectedNote?.created_at} updatedAt={selectedNote?.updated_at} />
          </div>
        )}
    </div>
  );
};