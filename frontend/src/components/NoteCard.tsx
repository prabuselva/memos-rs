import { memo } from 'react';

interface NoteCardProps {
  note: any;
  onClick: () => void;
  isSelected: boolean;
}

export const NoteCard = memo(({ note, onClick, isSelected }: NoteCardProps) => {
  const createdDate = new Date(note.created_at).toLocaleDateString();
  const tags = Array.isArray(note.tags) ? note.tags : (note.tags || '').split(',').map((t: string) => t.trim()).filter(Boolean);

  return (
    <div
      onClick={onClick}
      className={`cursor-pointer rounded-lg border p-3 md:p-4 transition-all duration-200 hover:shadow-md ${
        isSelected
          ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
          : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800'
      }`}
    >
      <h3 className="mb-2 line-clamp-1 font-semibold text-sm md:text-base text-gray-900 dark:text-gray-100">
        {note.title || 'Untitled'}
      </h3>
      <p className="mb-3 line-clamp-3 text-xs md:text-sm text-gray-600 dark:text-gray-400">
        {note.content || 'No content'}
      </p>
      <div className="flex items-center justify-between text-xs text-gray-500 dark:text-gray-500">
        <span className="text-[10px] md:text-xs">{createdDate}</span>
        {tags.length > 0 && (
          <div className="flex gap-1">
            {tags.slice(0, 3).map((tag: string, idx: number) => (
              <span
                key={idx}
                className="rounded-full bg-gray-100 px-1.5 py-0.5 text-[10px] dark:bg-gray-700"
              >
                {tag}
              </span>
            ))}
            {tags.length > 3 && (
              <span className="rounded-full bg-gray-100 px-1.5 py-0.5 text-[10px] dark:bg-gray-700">
                +{tags.length - 3}
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
});

NoteCard.displayName = 'NoteCard';