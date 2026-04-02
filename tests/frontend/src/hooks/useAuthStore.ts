import { create } from 'zustand';
import { type User } from '../lib/api';

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  
  login: (token: string, user: User) => void;
  logout: () => void;
  register: (token: string, user: User) => void;
  setUser: (user: User | null) => void;
  setToken: (token: string | null) => void;
  setError: (error: string | null) => void;
  setLoading: (loading: boolean) => void;
  setAuthenticated: (authenticated: boolean) => void;
  clearAuth: () => void;
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  token: null,
  isAuthenticated: false,
  isLoading: false,
  error: null,
  
  login: (token, user) => set({ 
    user, 
    token, 
    isAuthenticated: true, 
    error: null 
  }),
  
  logout: () => {
    localStorage.removeItem('auth');
    set({ 
      user: null, 
      token: null, 
      isAuthenticated: false, 
      error: null 
    });
  },
  
  register: (token, user) => set({ 
    user, 
    token, 
    isAuthenticated: true, 
    error: null 
  }),
  
  setUser: (user) => set({ user }),
  
  setToken: (token) => set({ token }),
  
  setError: (error) => set({ error }),
  
  setLoading: (loading) => set({ isLoading: loading }),
  
  setAuthenticated: (authenticated) => set({ isAuthenticated: authenticated }),
  
  clearAuth: () => {
    localStorage.removeItem('auth');
    set({ 
      user: null, 
      token: null, 
      isAuthenticated: false, 
      error: null 
    });
  },
}));

export const saveAuthToLocalStorage = (token: string, user: User) => {
  const authData = {
    token,
    user,
    expiresAt: Date.now() + 7 * 24 * 60 * 60 * 1000,
  };
  localStorage.setItem('auth', JSON.stringify(authData));
};

export const loadAuthFromLocalStorage = (): { token: string | null, user: User | null } | null => {
  const authData = localStorage.getItem('auth');
  if (!authData) return null;
  
  try {
    const parsed = JSON.parse(authData);
    if (Date.now() > parsed.expiresAt) {
      localStorage.removeItem('auth');
      return null;
    }
    return { token: parsed.token, user: parsed.user };
  } catch {
    localStorage.removeItem('auth');
    return null;
  }
};

export const clearAuthFromLocalStorage = () => {
  localStorage.removeItem('auth');
};