import { useState, useEffect } from 'react';
import { useNoteStore } from '../hooks/useNoteStore';
import { notesApi, type CreateNoteRequest } from '../lib/api';
import { MarkdownPreview } from './MarkdownPreview';

export const NoteEditor = () => {
  const { selectedNoteId, notes, updateNote, deleteNote, selectNote, setError, setLoading } = useNoteStore();
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [tags, setTags] = useState<string[]>([]);
  const [tagInputValue, setTagInputValue] = useState('');
  const [isEditing, setIsEditing] = useState(false);
  const [isPreviewMode, setIsPreviewMode] = useState(false);

  const selectedNote = notes.find((n) => n.id === selectedNoteId);

  useEffect(() => {
    if (selectedNote) {
      setTitle(selectedNote.title);
      setContent(selectedNote.content || '');
      setTags(selectedNote.tags || []);
      setIsPreviewMode(true);
      setIsEditing(false);
    } else {
      setTitle('');
      setContent('');
      setTags([]);
      setIsPreviewMode(false);
      setIsEditing(false);
    }
  }, [selectedNote]);

  const handleUpdate = async () => {
    if (!selectedNoteId) return;

    setLoading(true);
    try {
      const noteData: CreateNoteRequest = {
        title,
        content,
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

  if (!selectedNoteId && !isEditing) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-gray-500">
        <p className="text-xl">Select a note to view</p>
        <p className="text-sm mt-2">or create a new one</p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col pt-4">
      <div className="mb-4 flex items-center justify-between px-3">
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
        <div className="ml-4 flex flex-wrap gap-2">
          {isPreviewMode ? (
            <button
              onClick={handleEdit}
              className="rounded-lg bg-yellow-600 px-3 py-1.5 text-sm text-white hover:bg-yellow-700"
            >
              <span className="hidden sm:inline">Edit</span>
              <span className="sm:hidden">✎</span>
            </button>
          ) : (
            <button
              onClick={handleUpdate}
              className="rounded-lg bg-green-600 px-3 py-1.5 text-sm text-white hover:bg-green-700"
            >
              <span className="hidden sm:inline">Update</span>
              <span className="sm:hidden">✓</span>
            </button>
          )}
          {selectedNote && !isPreviewMode && (
            <button
              onClick={handleDelete}
              className="rounded-lg bg-red-600 px-3 py-1.5 text-sm text-white hover:bg-red-700"
            >
              <span className="hidden sm:inline">Delete</span>
              <span className="sm:hidden">🗑</span>
            </button>
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
            </div>
          </div>
        </>
      ) : (
        <div className="flex-1 overflow-y-auto min-h-0 w-full pb-4">
          <div className="w-full max-w-full overflow-x-auto px-1">
            <MarkdownPreview content={content || ''} />
          </div>
        </div>
      )}
    </div>
  );
};