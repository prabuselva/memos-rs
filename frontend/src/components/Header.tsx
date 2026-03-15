import { useState, useEffect, useRef, useCallback } from 'react';
import { useNoteStore } from '../hooks/useNoteStore';
import debounce from 'lodash.debounce';

import { notesApi, importExportApi, api } from '../lib/api';
import { ImportProgressModal } from './ImportProgressModal';
import { UserMenu } from './UserMenu';
import { SearchResults } from './SearchResults';

interface HeaderProps {
  onNewNote: () => void;
  onMenuToggle: () => void;
  onOpenProfile: () => void;
}

export const Header = ({ onNewNote, onMenuToggle, onOpenProfile }: HeaderProps) => {
  const { setError, setNotes, searchQuery, setSearchQuery, searchResults, setSearchResults, setIsSearching, isSearching, selectNote } = useNoteStore();
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isActionsOpen, setIsActionsOpen] = useState(false);
  const [importProgress, setImportProgress] = useState<{current: number, total: number, currentFile: string, status: 'uploading' | 'success' | 'error', errorMessage?: string} | null>(null);
  const [isMobile, setIsMobile] = useState(false);
  const actionsRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const handleResize = () => {
      const mobile = window.innerWidth < 768;
      setIsMobile(mobile);
      if (mobile && !isSearchOpen) {
        setIsActionsOpen(false);
      }
    };

    handleResize();
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, [isSearchOpen]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (actionsRef.current && !actionsRef.current.contains(event.target as Node)) {
        setIsActionsOpen(false);
      }
      if (isSearchOpen && searchInputRef.current && !searchInputRef.current.contains(event.target as Node)) {
        setIsSearchOpen(false);
      }
    };

    const handleKeyDown = (e: Event) => {
      const keyboardEvent = e as KeyboardEvent;
      if (keyboardEvent.key === '/' && !keyboardEvent.metaKey && !keyboardEvent.ctrlKey && !keyboardEvent.altKey) {
        e.preventDefault();
        setIsSearchOpen(true);
        setTimeout(() => searchInputRef.current?.focus(), 100);
      }
      if (keyboardEvent.key === 'Escape' && isSearchOpen) {
        e.preventDefault();
        setIsSearchOpen(false);
        setSearchQuery('');
      }
    };

    if (isActionsOpen || isSearchOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleKeyDown);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isActionsOpen, isSearchOpen]);

  const performSearch = useCallback(
    debounce(async (query: string) => {
      if (!query.trim()) {
        setSearchResults([]);
        setIsSearching(false);
        return;
      }
      setIsSearching(true);
      try {
        const response = await notesApi.search(query);
        setSearchResults(response.data);
        setSelectedIndex(0);
      } catch (error) {
        console.error('Search failed:', error);
        setSearchResults([]);
      } finally {
        setIsSearching(false);
      }
    }, 300),
    [setSearchResults, setIsSearching]
  );

  useEffect(() => {
    performSearch(searchQuery);
  }, [searchQuery, performSearch]);

  const handleFileSelect = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;
    if (!files || files.length === 0) {
      setError('No files selected');
      return;
    }

    const validFiles: {file: File, content: string}[] = [];
    
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      
      if (file.name.endsWith('.note') || file.name.endsWith('.xml')) {
        try {
          const content = await new Promise<string>((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = () => resolve(reader.result as string);
            reader.onerror = () => reject(reader.error);
            reader.readAsText(file);
          });
          
          const trimmed = content.trim();
          if (trimmed.includes('<note')) {
            validFiles.push({ file, content });
          }
        } catch (error) {
          console.error(`Error reading file ${file.name}:`, error);
        }
      }
    }

    if (validFiles.length === 0) {
      setError('No valid Tomboy note files found in the selected directory.');
      return;
    }

    setImportProgress({
      current: 0,
      total: validFiles.length,
      currentFile: validFiles[0].file.name,
      status: 'uploading',
    });

    const importedNoteIds: string[] = [];
    let failedIndex = -1;
    let failedFileName = '';

    try {
      for (let i = 0; i < validFiles.length; i++) {
        setImportProgress({
          current: i,
          total: validFiles.length,
          currentFile: validFiles[i].file.name,
          status: 'uploading',
        });

        try {
          const response = await api.post<{imported: number, note_ids: string[]}>(
            '/import/tomboy/xml',
            { xml: validFiles[i].content }
          );

          if (response.data.note_ids && response.data.note_ids.length > 0) {
            importedNoteIds.push(...response.data.note_ids);
          }
        } catch (error) {
          failedIndex = i;
          failedFileName = validFiles[i].file.name;
          throw error;
        }
      }

      const updatedNotes = await notesApi.getAll();
      setNotes(updatedNotes.data);
      setError(null);
      setImportProgress({
        current: validFiles.length,
        total: validFiles.length,
        currentFile: '',
        status: 'success',
      });

    } catch {
      if (importedNoteIds.length > 0) {
        try {
          await importExportApi.rollbackImport(importedNoteIds);
        } catch (rollbackError) {
          console.error('Rollback failed:', rollbackError);
        }
      }

      setError(`Failed to import ${failedFileName || validFiles[0].file.name}. All imports rolled back.`);
      setImportProgress({
        current: failedIndex >= 0 ? failedIndex : 0,
        total: validFiles.length,
        currentFile: failedFileName || validFiles[0].file.name,
        status: 'error',
        errorMessage: `Failed to import ${failedFileName || validFiles[0].file.name}. All imports rolled back.`,
      });
    }
  };

  return (
    <>
      <div className="relative flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex-shrink-0">
        <div className="flex items-center gap-2 flex-shrink-0">
          <button
            onClick={onMenuToggle}
            className="flex-shrink-0 p-2 rounded-lg bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
            aria-label="Toggle Sidebar"
          >
            <svg className="w-6 h-6 text-gray-700 dark:text-gray-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
          {!isMobile && (
            <h1 className="text-xl font-bold text-blue-600 dark:text-blue-400">Memos RS</h1>
          )}
        </div>
        
        <div className="flex-1 px-3 md:px-6 flex justify-center">
          <div className="relative w-full max-w-md" ref={searchInputRef}>
            <button
              onClick={() => {
                setIsSearchOpen(!isSearchOpen);
                if (!isSearchOpen) {
                  setTimeout(() => searchInputRef.current?.focus(), 100);
                }
              }}
              className={`flex items-center w-full px-3 md:px-4 py-2.5 rounded-lg border transition-all ${
                isSearchOpen
                  ? 'border-blue-500 bg-white dark:bg-gray-800 shadow-md'
                  : 'border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 hover:bg-gray-100 dark:hover:bg-gray-600'
              }`}
              aria-label="Search notes"
              title="Search notes (Press /)"
            >
              <svg className="w-4 h-4 md:w-5 md:h-5 text-gray-500 dark:text-gray-400 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <span className="flex-1 text-sm text-gray-700 dark:text-gray-300 text-left truncate">
                {isSearchOpen ? '' : isMobile ? '' : 'Search notes...'}
              </span>
              {isSearching && !isSearchOpen && (
                <svg className="w-3 h-3 md:w-4 md:h-4 text-gray-400 animate-spin flex-shrink-0" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
              )}
              {isSearchOpen && (
                <input
                  ref={searchInputRef}
                  type="text"
                  placeholder=""
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && isMobile) {
                      e.preventDefault();
                      onMenuToggle();
                    }
                  }}
                  className="w-full px-2 py-1 bg-transparent border-none outline-none text-sm text-gray-900 dark:text-gray-100"
                  autoFocus
                />
              )}
              <button
                onClick={() => {
                  setSearchQuery('');
                  setIsSearchOpen(false);
                }}
                className={`ml-1 md:ml-2 p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600 ${isSearchOpen ? 'opacity-100' : 'opacity-0'}`}
              >
                <svg className="w-3 h-3 md:w-4 md:h-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </button>
            {isSearchOpen && (
              <div className="absolute top-full mt-2 w-full rounded-lg bg-white dark:bg-gray-800 shadow-xl ring-1 ring-black ring-opacity-5 dark:ring-white dark:ring-opacity-10 z-50 overflow-hidden">
                <SearchResults
                  results={searchResults}
                  query={searchQuery}
                  onSelectNote={(id) => {
                    const note = searchResults.find(n => n.id === id);
                    if (note) {
                      selectNote(note.id);
                    }
                  }}
                  onClose={() => setIsSearchOpen(false)}
                  selectedIndex={selectedIndex}
                  setSelectedIndex={setSelectedIndex}
                />
              </div>
            )}
          </div>
        </div>
        
        <div className="flex items-center gap-2 md:gap-3 flex-shrink-0">
          <div className="relative" ref={actionsRef}>
            <button
              onClick={() => setIsActionsOpen(!isActionsOpen)}
              className="flex items-center justify-center w-9 h-9 md:w-10 md:h-10 rounded-lg bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
              aria-label="Actions"
            >
              <svg className="w-5 h-5 md:w-6 md:h-6 text-gray-700 dark:text-gray-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
              </svg>
            </button>
            {isActionsOpen && (
              <div className="absolute right-0 top-full mt-1 w-56 md:w-64 bg-white dark:bg-gray-800 rounded-lg shadow-lg z-50 border border-gray-200 dark:border-gray-700 overflow-hidden">
                <div className="py-1">
                  <button
                    onClick={() => {
                      onNewNote();
                      setIsActionsOpen(false);
                    }}
                    className="flex items-center w-full px-4 py-2 text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700"
                  >
                    <span className="mr-2">+</span>
                    <span className="text-sm">New Note</span>
                  </button>
                  <label className="flex items-center w-full px-4 py-2 text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer">
                    <input
                      type="file"
                      multiple
                      // @ts-expect-error webkitdirectory is not recognized by TypeScript
                      webkitdirectory=""
                      className="hidden"
                      onChange={handleFileSelect}
                    />
                    <span className="ml-2 text-sm">Import Notes</span>
                  </label>
                </div>
              </div>
            )}
          </div>
          <UserMenu onOpenProfile={onOpenProfile} />
        </div>
      </div>
      <ImportProgressModal
        isOpen={importProgress !== null}
        total={importProgress?.total || 0}
        current={importProgress?.current || 0}
        currentFile={importProgress?.currentFile || ''}
        status={importProgress?.status || 'uploading'}
        errorMessage={importProgress?.errorMessage}
      />
    </>
  );
};