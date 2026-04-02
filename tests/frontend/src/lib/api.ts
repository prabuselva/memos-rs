import axios from 'axios';
import { useAuthStore } from '../hooks/useAuthStore';

const getApiBaseUrl = () => {
  const protocol = window.location.protocol;
  const host = window.location.host;
  return `${protocol}//${host}/api/v1`;
};

const API_BASE_URL = import.meta.env.VITE_API_URL || getApiBaseUrl();

export const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

api.interceptors.request.use((config) => {
  const { token } = useAuthStore.getState();
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      console.warn('Unauthorized - clearing auth');
      useAuthStore.getState().clearAuth();
      localStorage.removeItem('auth');
    }
    return Promise.reject(error);
  }
);

export interface Note {
  id: string;
  title: string;
  content: string;
  content_html?: string | null;
  notebook_id?: string | null;
  parent_id: string | null;
  created_at: string;
  updated_at: string;
  is_favorite: boolean;
  is_archived: boolean;
  tags: string[];
  metadata: Record<string, any>;
  user_id?: string | null;
}

export interface Notebook {
  id: string;
  name: string;
  parent_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface Tag {
  id: string;
  name: string;
  color?: string;
}

export interface CreateNoteRequest {
  title: string;
  content: string;
  notebook_id?: string;
  parent_id?: string;
  tags?: string[];
  is_favorite?: boolean;
  is_archived?: boolean;
}

export interface UpdateNoteRequest {
  title?: string;
  content?: string;
  parent_id?: string;
  tags?: string[];
  is_favorite?: boolean;
  is_archived?: boolean;
}

export interface User {
  id: string;
  username: string;
  email: string;
  created_at: string;
}

export interface LoginRequest {
  username: string;
  password: string;
  remember_me?: boolean;
}

export interface LoginResponse {
  token: string;
  user: User;
  expires_at: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
  password_confirm: string;
}

export interface RegisterResponse {
  user: User;
  token: string;
  expires_at: string;
}

export interface PasswordResetRequest {
  email: string;
}

export interface PasswordResetConfirm {
  token: string;
  password: string;
  password_confirm: string;
}

export interface UpdateProfileRequest {
  username?: string;
  email?: string;
  current_password?: string;
  new_password?: string;
}



export const importExportApi = {
  exportTomboy: () => api.get<string>('/export/tomboy'),
  rollbackImport: (noteIds: string[]) => 
    api.post<{deleted: number}>('/import/tomboy/rollback', { 
      note_ids: noteIds 
    }),
};

export const notebooksApi = {
  getAll: () => api.get<Notebook[]>('/notebooks'),
  getById: (id: string) => api.get<Notebook>(`/notebooks/${id}`),
  create: (data: { name: string; parent_id?: string }) => api.post<Notebook>('/notebooks', data),
  update: (id: string, data: { name: string; parent_id?: string }) => api.put<Notebook>(`/notebooks/${id}`, data),
  delete: (id: string) => api.delete(`/notebooks/${id}`),
  getNotes: (id: string, limit?: number, offset?: number) => 
    api.get<Note[]>('/notebooks/' + id + '/notes', { params: { limit, offset } }),
  reorder: (data: { items: Array<{ id: string; type: 'notebook' | 'note'; parent_id?: string; order?: number }> }) => 
    api.put('/notebooks/reorder', data),
  getFlat: () => api.get<Notebook[]>('/notebooks/tree'),
  getTreeWithNotes: () => api.get<{ folders: Array<{ id: string; name: string; parent_id: string | null; children: any[]; notes: any[] }> }>('/notebooks/tree'),
};

export const notesApi = {
  getAll: () => api.get<Note[]>('/notes'),
  getById: (id: string) => api.get<Note>(`/notes/${id}`),
  create: (data: CreateNoteRequest) => api.post<Note>('/notes', data),
  update: (id: string, data: UpdateNoteRequest) => api.put<Note>(`/notes/${id}`, data),
  delete: (id: string) => api.delete(`/notes/${id}`),
  getContent: (id: string) => api.get<string>(`/notes/${id}/content`),
  search: (query: string) => api.get<Note[]>('/notes/search', { params: { q: query } }),
  vectorSearch: (query: string, limit?: number) => 
    api.get<Note[]>('/notes/vector-search', { params: { q: query, limit } }),
  reorder: (data: { items: Array<{ id: string; type: 'notebook' | 'note'; parent_id?: string; order?: number }> }) => 
    api.put('/notes/reorder', data),
  bulkDelete: (data: { ids: string[]; type: 'notebook' | 'note' }) => 
    api.post('/bulk-delete', data),
};

export const tagsApi = {
  getAll: () => api.get<Tag[]>('/tags'),
};

export const authApi = {
  login: (data: LoginRequest) => api.post<LoginResponse>('/login', data),
  register: (data: RegisterRequest) => api.post<RegisterResponse>('/register', data),
  logout: () => api.post('/logout'),
  refresh: (token: string) => api.post<LoginResponse>('/refresh', { token }),
  me: () => api.get<User>('/me'),
  updateProfile: (data: UpdateProfileRequest) => api.put<User>('/profile', data),
  requestPasswordReset: (data: PasswordResetRequest) => api.post('/request-password-reset', data),
  resetPassword: (data: PasswordResetConfirm) => api.post('/reset-password', data),
  updateSearchMode: (searchMode: string) => api.put<{ search_mode: string }>('/profile/search-mode', { search_mode: searchMode }),
  getSearchMode: () => api.get<{ search_mode: string }>('/profile/search-mode'),
  updateLLMSettings: (settings: any) => api.put<any>('/profile/llm-settings', { llm_settings: settings }),
  getLLMSettings: () => api.get<any>('/profile/llm-settings'),
  testLLMConnection: (settings: any) => api.post<any>('/profile/llm-settings/test', settings),
};

export interface VectorDBStatus {
  enabled: boolean;
  available: boolean;
  message: string;
}

export const vectorDbApi = {
  getStatus: async (): Promise<VectorDBStatus> => {
    try {
      const response = await api.get<VectorDBStatus>('/vector-db/status');
      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error) && error.response?.status === 404) {
        return { enabled: false, available: false, message: 'Vector DB not configured' };
      }
      throw error;
    }
  },
};

export default api;
export { chatApi } from './chatApi';