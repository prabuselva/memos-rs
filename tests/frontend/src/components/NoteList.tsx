import { useNoteStore } from '../hooks/useNoteStore';
import { NoteCard } from './NoteCard';

interface NoteListProps {
  onSelectNote?: (noteId: string) => void;
}

export const NoteList = ({ onSelectNote }: NoteListProps) => {
  const { notes, selectNote, selectedNoteId, searchQuery, selectedNotebook } = useNoteStore();

  const filteredNotes = notes.filter(
    (note) =>
      !note.is_archived &&
      (!selectedNotebook || note.notebook_id === selectedNotebook) &&
      (note.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      note.content.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  const handleSelectNote = (id: string) => {
    selectNote(id);
    if (onSelectNote) {
      onSelectNote(id);
    }
  };

  return (
    <div className="flex-1 overflow-y-auto">
      {filteredNotes.length === 0 ? (
        <div className="flex h-full flex-col items-center justify-center text-gray-500 p-4">
          <p className="text-sm">No notes found</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 p-4">
          {filteredNotes.map((note) => (
            <NoteCard
              key={note.id}
              note={note}
              onClick={() => handleSelectNote(note.id)}
              isSelected={selectedNoteId === note.id}
            />
          ))}
        </div>
      )}
    </div>
  );
};