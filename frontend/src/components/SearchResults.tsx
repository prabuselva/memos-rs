import { useRef, useEffect } from 'react';
import type { Note } from '../lib/api';

interface SearchResultsProps {
  results: Note[];
  query: string;
  onSelectNote: (noteId: string) => void;
  onClose: () => void;
  selectedIndex: number;
  setSelectedIndex: (index: number) => void;
}

export const SearchResults = ({
  results,
  query,
  onSelectNote,
  onClose,
  selectedIndex,
  setSelectedIndex,
}: SearchResultsProps) => {
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        onClose();
      }
    };

    if (results.length > 0) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [onClose, results.length]);

  useEffect(() => {
    if (selectedIndex > 0 && dropdownRef.current) {
      const activeElement = dropdownRef.current.querySelector(
        `[data-index="${selectedIndex}"]`
      ) as HTMLElement;
      if (activeElement) {
        activeElement.scrollIntoView({ block: 'nearest' });
      }
    }
  }, [selectedIndex]);

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
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  };

  const highlightText = (text: string, query: string) => {
    if (!query) return text;
    const regex = new RegExp(`(${query})`, 'gi');
    const parts = text.split(regex);
    return parts.map((part, i) =>
      regex.test(part) ? (
        <span key={i} className="bg-yellow-200 dark:bg-yellow-600/50 font-semibold">
          {part}
        </span>
      ) : (
        <span key={i}>{part}</span>
      )
    );
  };

  const getSnippet = (content: string, query: string): string => {
    if (!query || content.length <= 150) return content;
    const index = content.toLowerCase().indexOf(query.toLowerCase());
    if (index === -1) return content.substring(0, 150) + '...';
    const start = Math.max(0, index - 50);
    const end = Math.min(content.length, index + 100);
    const snippet = content.substring(start, end);
    return (start > 0 ? '...' : '') + snippet + (end < content.length ? '...' : '');
  };

  if (results.length === 0 && !query) {
    return null;
  }

  return (
    <div
      ref={dropdownRef}
      className="absolute left-0 top-full mt-2 w-96 max-h-96 overflow-y-auto rounded-lg bg-white dark:bg-gray-800 shadow-xl ring-1 ring-black ring-opacity-5 dark:ring-white dark:ring-opacity-10 z-50"
      tabIndex={-1}
      onKeyDown={handleKeyDown}
    >
      <div className="py-1">
        {results.length === 0 ? (
          <div className="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
            No notes found
          </div>
        ) : (
          results.map((note, index) => (
            <div
              key={note.id}
              data-index={index}
              className={`flex items-start gap-3 px-4 py-3 cursor-pointer transition-colors ${
                index === selectedIndex
                  ? 'bg-blue-50 dark:bg-blue-900/30'
                  : 'hover:bg-gray-50 dark:hover:bg-gray-700/50'
              }`}
              onClick={() => {
                onSelectNote(note.id);
                onClose();
              }}
            >
              <div className="mt-0.5">
                <svg
                  className="h-5 w-5 text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                  />
                </svg>
              </div>
              <div className="flex-1 min-w-0">
                <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-1">
                  {highlightText(note.title || 'Untitled', query)}
                </h3>
                {note.content && (
                  <p className="text-xs text-gray-500 dark:text-gray-400 line-clamp-2">
                    {highlightText(getSnippet(note.content, query), query)}
                  </p>
                )}
                <div className="flex items-center gap-2 mt-2">
                  <span className="text-[10px] text-gray-400">
                    {new Date(note.updated_at).toLocaleDateString()}
                  </span>
                  {note.tags && note.tags.length > 0 && (
                    <div className="flex gap-1">
                      {note.tags.slice(0, 2).map((tag, i) => (
                        <span
                          key={i}
                          className="text-[9px] px-1.5 py-0.5 bg-gray-100 dark:bg-gray-700 rounded-full text-gray-600 dark:text-gray-300"
                        >
                          {tag}
                        </span>
                      ))}
                      {note.tags.length > 2 && (
                        <span className="text-[9px] text-gray-400">
                          +{note.tags.length - 2}
                        </span>
                      )}
                    </div>
                  )}
                </div>
              </div>
            </div>
          ))
        )}
      </div>
      <div className="border-t border-gray-100 dark:border-gray-700 px-4 py-2 text-xs text-gray-400 flex items-center justify-between">
        <span>
          {results.length} result{results.length !== 1 && 's'}
        </span>
        <div className="flex gap-3">
          <span className="flex items-center gap-1">
            <kbd className="px-1 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">↑↓</kbd>
            to navigate
          </span>
          <span className="flex items-center gap-1">
            <kbd className="px-1 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">↵</kbd>
            to open
          </span>
          <span className="flex items-center gap-1">
            <kbd className="px-1 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">esc</kbd>
            to close
          </span>
        </div>
      </div>
    </div>
  );
};