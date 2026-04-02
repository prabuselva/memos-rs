import { create } from 'zustand';
import { type Note, type Tag, type Notebook } from '../lib/api';

export type SearchMode = 'sql' | 'vector';

interface NoteState {
  notes: Note[];
  selectedNoteId: string | null;
  notebooks: Notebook[];
  tags: Tag[];
  searchQuery: string;
  searchResults: Note[];
  selectedTag: string | null;
  selectedNotebook: string | null;
  sidebarMode: 'list' | 'tree';
  searchMode: SearchMode;
  isDarkMode: boolean;
  isLoading: boolean;
  isSearching: boolean;
  error: string | null;
  isRearrangeModalOpen: boolean;
  
  setNotes: (notes: Note[]) => void;
  addNote: (note: Note) => void;
  updateNote: (note: Note) => void;
  deleteNote: (id: string) => void;
  selectNote: (id: string | null) => void;
  setSelectedTag: (tag: string | null) => void;
  setSelectedNotebook: (notebook: string | null) => void;
  setNotebooks: (notebooks: Notebook[]) => void;
  setSidebarMode: (mode: 'list' | 'tree') => void;
  setSearchQuery: (query: string) => void;
  setSearchResults: (results: Note[]) => void;
  setIsSearching: (searching: boolean) => void;
  setSearchMode: (mode: SearchMode) => void;
  setDarkMode: (isDark: boolean) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  setIsRearrangeModalOpen: (isOpen: boolean) => void;
}

const detectDarkMode = () => {
  if (typeof window !== 'undefined' && window.matchMedia) {
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  }
  return false;
};

export const useNoteStore = create<NoteState>((set) => ({
  notes: [],
  selectedNoteId: null,
  notebooks: [],
  tags: [],
  searchQuery: '',
  searchResults: [],
  selectedTag: null,
  selectedNotebook: null,
  sidebarMode: 'tree',
  searchMode: 'sql',
  isDarkMode: detectDarkMode(),
  isLoading: false,
  isSearching: false,
  error: null,
  isRearrangeModalOpen: false,
  
  setNotes: (notes) => set({ notes }),
  addNote: (note) => set((state) => ({ notes: [note, ...state.notes] })),
  updateNote: (note) => set((state) => ({
    notes: state.notes.map((n) => (n.id === note.id ? note : n)),
  })),
  deleteNote: (id) => set((state) => ({
    notes: state.notes.filter((n) => n.id !== id),
    selectedNoteId: state.selectedNoteId === id ? null : state.selectedNoteId,
  })),
  selectNote: (id) => set({ selectedNoteId: id }),
  setSelectedTag: (tag) => set({ selectedTag: tag }),
  setSelectedNotebook: (notebook) => set({ selectedNotebook: notebook }),
   setNotebooks: (notebooks) => set({ notebooks }),
  setSidebarMode: (mode) => set({ sidebarMode: mode }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setSearchResults: (results) => set({ searchResults: results }),
  setIsSearching: (searching) => set({ isSearching: searching }),
  setSearchMode: (mode) => set({ searchMode: mode }),
  setDarkMode: (isDark) => set({ isDarkMode: isDark }),
  setLoading: (loading) => set({ isLoading: loading }),
  setError: (error) => set({ error }),
   setIsRearrangeModalOpen: (isOpen) => set({ isRearrangeModalOpen: isOpen }),
}));