import { create } from 'zustand';
import type { Note } from '../lib/api';

export type ChatMode = 'rag' | 'search' | 'chat';

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
  sources?: string[];
  relatedNotes?: Note[];
  references?: Reference[];
  searchMetadata?: SearchMetadata;
}

export interface Reference {
  id: number;
  note_id: string;
  title?: string;
  content_snippet: string;
  score: number;
  distance: number;
  used_in_response: boolean;
  created_at?: string;
  updated_at?: string;
  tags?: string[];
}

export interface SearchMetadata {
  query: string;
  vector_search_time_ms: number;
  llm_generation_time_ms: number;
  total_tokens: number;
  retrieved_count: number;
  filtered_count: number;
  hybrid_search: boolean;
  model: string;
}

export interface ChatSession {
  id: string;
  title: string;
  messages: ChatMessage[];
  createdAt: string;
  updatedAt: string;
}

export interface ChatState {
  messages: ChatMessage[];
  currentSession: ChatSession | null;
  chatHistory: ChatSession[];
  isChatOpen: boolean;
  isSearching: boolean;
  searchResults: Note[];
  currentMode: ChatMode;
  userQuery: string;
  selectedContextNotes: string[];
  error: string | null;
  
  openChat: () => void;
  closeChat: () => void;
  setMode: (mode: ChatMode) => void;
  setUserQuery: (query: string) => void;
  addMessage: (message: ChatMessage) => void;
  createSession: () => void;
  loadSession: (sessionId: string) => void;
  updateSession: (sessionId: string, updates: Partial<ChatSession>) => void;
  setSearchResults: (results: Note[]) => void;
  addContextNote: (noteId: string) => void;
  removeContextNote: (noteId: string) => void;
  setError: (error: string | null) => void;
  clearCurrentSession: () => void;
}

const generateId = () => Math.random().toString(36).substring(2, 15);

export const useChatStore = create<ChatState>((set, get) => ({
  messages: [],
  currentSession: null,
  chatHistory: [],
  isChatOpen: false,
  isSearching: false,
  searchResults: [],
  currentMode: 'rag',
  userQuery: '',
  selectedContextNotes: [],
  error: null,
  
  openChat: () => set({ isChatOpen: true, error: null }),
  closeChat: () => set({ isChatOpen: false }),
  
  setMode: (mode) => set((state) => {
     const newMessages = mode === 'chat' ? state.messages : [];
     return { currentMode: mode, userQuery: '', messages: newMessages };
   }),
  
  setUserQuery: (query) => set({ userQuery: query }),
  
  addMessage: (message) => set((state) => {
    const messages = [...state.messages, message];
    return {
      messages,
      currentSession: state.currentSession
        ? {
            ...state.currentSession,
            messages,
            updatedAt: new Date().toISOString(),
            title: state.messages.length === 0 && message.role === 'user'
              ? message.content.slice(0, 50) + (message.content.length > 50 ? '...' : '')
              : state.currentSession.title,
          }
        : null,
    };
  }),
  
  createSession: () => {
    const newSession: ChatSession = {
      id: generateId(),
      title: 'New Chat',
      messages: [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    set({
      currentSession: newSession,
      messages: [],
      chatHistory: [newSession, ...get().chatHistory],
    });
  },
  
  loadSession: (sessionId) => {
    const session = get().chatHistory.find((s) => s.id === sessionId);
    if (session) {
      set({ currentSession: session, messages: session.messages });
    }
  },
  
  updateSession: (sessionId, updates) =>
    set((state) => ({
      chatHistory: state.chatHistory.map((session) =>
        session.id === sessionId ? { ...session, ...updates } : session
      ),
      currentSession:
        state.currentSession?.id === sessionId
          ? { ...state.currentSession, ...updates }
          : state.currentSession,
    })),
  
  setSearchResults: (results) => set({ searchResults: results }),
  
  addContextNote: (noteId) =>
    set((state) => ({
      selectedContextNotes: [...state.selectedContextNotes, noteId],
    })),
  
  removeContextNote: (noteId) =>
    set((state) => ({
      selectedContextNotes: state.selectedContextNotes.filter((id) => id !== noteId),
    })),
  
  setError: (error) => set({ error }),
   
   clearCurrentSession: () => {
     const currentSession = get().currentSession;
     if (currentSession) {
       set({
         messages: [],
         currentSession: {
           ...currentSession,
           messages: [],
           updatedAt: new Date().toISOString(),
         },
         chatHistory: get().chatHistory.map((session) =>
           session.id === currentSession.id
             ? { ...session, messages: [] }
             : session
         ),
       });
     }
   },
}));
