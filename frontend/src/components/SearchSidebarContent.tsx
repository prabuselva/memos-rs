import { useState } from 'react';
import { NoteCard } from './NoteCard';
import { useNoteStore } from '../hooks/useNoteStore';

interface SearchSidebarContentProps {
  query: string;
  onSelectNote: (noteId: string) => void;
}

export const SearchSidebarContent = ({ query, onSelectNote }: SearchSidebarContentProps) => {
  const { searchResults: storeResults } = useNoteStore();
  const [selectedIndex, setSelectedIndex] = useState(0);

  const results = storeResults;

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex(Math.min(selectedIndex + 1, results.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex(Math.max(selectedIndex - 1, 0));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (results[selectedIndex]) {
        onSelectNote(results[selectedIndex].id);
      }
    }
  };

  return (
    <div
      className="flex-1 overflow-y-auto p-4"
      onKeyDown={handleKeyDown}
      tabIndex={0}
    >
      {query.trim() === '' ? (
        <div className="flex flex-col items-center justify-center h-full text-gray-500 p-4">
          <p className="text-sm">Type to search notes</p>
        </div>
      ) : results.length === 0 ? (
        <div className="flex flex-col items-center justify-center h-full text-gray-500 p-4">
          <p className="text-sm">No notes found</p>
        </div>
      ) : (
        <div className="space-y-3">
          {results.map((note, index) => (
            <NoteCard
              key={note.id}
              note={note}
              onClick={() => {
                onSelectNote(note.id);
              }}
              isSelected={index === selectedIndex}
            />
          ))}
        </div>
      )}
    </div>
  );
};
