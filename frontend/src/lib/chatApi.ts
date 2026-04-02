import { api } from './api';
import type { Note } from './api';

export interface BackendChatResponse {
  response: string;
  context_notes: BackendNoteReference[];
  references?: Reference[];
  search_metadata?: SearchMetadata;
  model: string;
}

export interface BackendNoteReference {
  id: number;
  note_id?: string;
  title: string;
  content: string;
  score: number;
  distance?: number;
  user_id?: string;
  created_at?: string;
  updated_at?: string;
  tags?: string[];
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

export interface RagSearchRequest {
  query: string;
  limit?: number;
  use_hybrid_search?: boolean;
  max_context_tokens?: number;
  mode?: 'rag' | 'search' | 'chat';
}

export interface ChatMessageHistory {
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
}

export interface ChatRequest {
  query: string;
  mode?: 'rag' | 'search' | 'chat';
  session_id?: string;
  context_notes?: string[];
  history?: ChatMessageHistory[];
}

export interface SourceInfo {
  id: string;
  title: string;
}

export interface ChatResponse {
  answer: string;
  sources?: SourceInfo[];
  session_id: string;
  references?: Reference[];
  searchMetadata?: SearchMetadata;
}

export interface SearchResponse {
  results: Note[];
  answer?: string;
  searchMetadata?: SearchMetadata;
}

export const chatApi = {
  /**
   * Send a chat query to the backend
   */
  chat: async (query: string, contextNotes?: string[], mode: 'rag' | 'search' | 'chat' = 'rag', history?: ChatMessageHistory[]): Promise<ChatResponse> => {
    const response = await api.post<BackendChatResponse>('/chat/query', {
      query,
      mode: mode,
      context_note_ids: contextNotes?.map((id) => parseInt(id)),
      history,
    });
    return {
      answer: response.data.response,
      sources: response.data.context_notes?.map((n) => ({
        id: n.id.toString(),
        title: n.title || `Note ${n.id}`,
      })),
      session_id: Date.now().toString(),
      references: response.data.references,
      searchMetadata: response.data.search_metadata,
    };
  },
  
  /**
   * Search notes using natural language with RAG
   */
  search: async (query: string, mode: 'rag' | 'search' | 'chat' = 'rag'): Promise<SearchResponse> => {
    const response = await api.post<BackendChatResponse>('/chat/query', {
      query,
      mode: mode,
      context_note_ids: [],
    });
    return {
      results: [],
      answer: response.data.response,
      searchMetadata: response.data.search_metadata,
    };
  },
  
  /**
   * Perform RAG search with custom parameters
   */
  ragSearch: async (params: RagSearchRequest): Promise<BackendChatResponse> => {
    const response = await api.post<BackendChatResponse>('/chat/search-rag', params);
    return response.data;
  },
  
  /**
   * Get chat session history
   */
  getHistory: async (): Promise<ChatResponse[]> => {
    const response = await api.get<ChatResponse[]>('/chat/history');
    return response.data;
  },
  
  /**
   * Create a new chat session
   */
  createSession: async (): Promise<{ session_id: string }> => {
    const response = await api.post<{ session_id: string }>('/chat/session');
    return response.data;
  },
  
  /**
   * Get notes by IDs
   */
  getNotesByIds: async (noteIds: string[]): Promise<Note[]> => {
    if (noteIds.length === 0) return [];
    
    const response = await api.post<Note[]>('/chat/notes', { note_ids: noteIds });
    return response.data;
  },
};
