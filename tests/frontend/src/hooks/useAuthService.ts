import { useEffect } from 'react';
 import { useAuthStore, loadAuthFromLocalStorage, saveAuthToLocalStorage, clearAuthFromLocalStorage } from './useAuthStore';
 import { useNoteStore } from './useNoteStore';
 import { authApi } from '../lib/api';

 export const useInitializeAuth = () => {
   const { setUser, setToken, setLoading, setError, setAuthenticated } = useAuthStore();
   const { setSearchMode } = useNoteStore();
   
   useEffect(() => {
     const initializeAuth = async () => {
       setLoading(true);
       
       try {
         const storedAuth = loadAuthFromLocalStorage();
         
         if (storedAuth && storedAuth.token) {
           try {
             setToken(storedAuth.token);
             setUser(storedAuth.user);
             setAuthenticated(true);
             const response = await authApi.me();
             setUser(response.data);
             saveAuthToLocalStorage(storedAuth.token, response.data);
             
             // Load search mode from backend
             const searchModeResponse = await authApi.getSearchMode();
             if (searchModeResponse.data) {
               setSearchMode(searchModeResponse.data.search_mode as 'sql' | 'vector');
             }
           } catch {
             clearAuthFromLocalStorage();
           }
         }
       } catch (error: unknown) {
         console.error('Auth initialization failed:', error);
       } finally {
         setLoading(false);
       }
     };
     
     initializeAuth();
   }, [setUser, setToken, setLoading, setError, setAuthenticated, setSearchMode]);
 };

export const useAuthService = () => {
  const { user, isAuthenticated, setLoading, setError, clearAuth } = useAuthStore();
  
  const login = async (username: string, password: string) => {
    setLoading(true);
    try {
      const response = await authApi.login({ username, password });
      saveAuthToLocalStorage(response.data.token, response.data.user);
      return { success: true, user: response.data.user };
    } catch (error: unknown) {
      const errorMessage = error instanceof Error ? error.message : 'Login failed';
      setError(errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      setLoading(false);
    }
  };
  
  const register = async (username: string, email: string, password: string, confirmPassword: string) => {
    setLoading(true);
    try {
      const response = await authApi.register({ username, email, password, password_confirm: confirmPassword });
      saveAuthToLocalStorage(response.data.token, response.data.user);
      return { success: true, user: response.data.user };
    } catch (error: unknown) {
      const errorMessage = error instanceof Error ? error.message : 'Registration failed';
      setError(errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      setLoading(false);
    }
  };
  
  const logout = async () => {
    setLoading(true);
    try {
      await authApi.logout();
    } catch (error: unknown) {
      console.error('Logout failed:', error);
    } finally {
      clearAuth();
      clearAuthFromLocalStorage();
    }
  };
  
  const requestPasswordReset = async (email: string) => {
    setLoading(true);
    try {
      await authApi.requestPasswordReset({ email });
      return { success: true };
    } catch (error: unknown) {
      const errorMessage = error instanceof Error ? error.message : 'Password reset request failed';
      setError(errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      setLoading(false);
    }
  };
  
  const resetPassword = async (token: string, password: string, confirmPassword: string) => {
    setLoading(true);
    try {
      await authApi.resetPassword({ token, password, password_confirm: confirmPassword });
      return { success: true };
    } catch (error: unknown) {
      const errorMessage = error instanceof Error ? error.message : 'Password reset failed';
      setError(errorMessage);
      return { success: false, error: errorMessage };
    } finally {
      setLoading(false);
    }
  };
  
  return {
    user,
    isAuthenticated,
    login,
    register,
    logout,
    requestPasswordReset,
    resetPassword,
  };
};