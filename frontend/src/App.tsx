import { useEffect, useCallback, useState } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { useNoteStore } from './hooks/useNoteStore';
import { useInitializeData } from './hooks/useData';
import { useInitializeAuth } from './hooks/useAuthService';
import { useAuthStore } from './hooks/useAuthStore';
import { NoteList } from './components/NoteList';
import { NoteEditor } from './components/NoteEditor';
import { Header } from './components/Header';
import { Login } from './components/Login';
import { Register } from './components/Register';
import { ForgotPassword } from './components/ForgotPassword';
import { ResetPassword } from './components/ResetPassword';
import { Profile } from './components/Profile';
import { notesApi } from './lib/api';

function AppContent() {
  const { isDarkMode, addNote, selectNote, setError } = useNoteStore();
  const [isMobile, setIsMobile] = useState(false);
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [isProfileOpen, setIsProfileOpen] = useState(false);
  
  const openProfile = () => {
    setIsProfileOpen(true);
    setError(null);
  };

  useEffect(() => {
    if (isDarkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [isDarkMode]);

  useEffect(() => {
    const handleResize = () => {
      setIsMobile(window.innerWidth < 768);
      if (window.innerWidth >= 768) {
        setIsSidebarOpen(true);
      } else {
        setIsSidebarOpen(false);
      }
    };

    handleResize();
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, []);

  const handleNewNote = useCallback(async () => {
    try {
      const response = await notesApi.create({
        title: '',
        content: '',
        tags: [],
        is_favorite: false,
        is_archived: false,
      });
      addNote(response.data);
      selectNote(response.data.id);
      if (isMobile) {
        setIsSidebarOpen(false);
      }
    } catch {
      setError('Failed to create note');
    }
  }, [addNote, selectNote, setError, isMobile]);

  const handleSelectNote = useCallback((noteId: string) => {
    selectNote(noteId);
    if (isMobile) {
      setIsSidebarOpen(false);
    }
  }, [selectNote, isMobile]);

  const handleProfileClose = () => {
    setIsProfileOpen(false);
  };

  return (
    <div className={`flex h-screen flex-col ${isDarkMode ? 'dark' : ''}`}>
      <Header onNewNote={handleNewNote} onMenuToggle={() => setIsSidebarOpen(!isSidebarOpen)} onOpenProfile={openProfile} />
      <div className="flex-1 flex overflow-hidden bg-gray-50 dark:bg-gray-900 relative">
        {isMobile && (
          <div
            className={`fixed inset-0 bg-black bg-opacity-50 z-40 transition-opacity duration-300 ${
              isSidebarOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'
            }`}
            onClick={() => setIsSidebarOpen(false)}
          />
        )}
        <div
          className={`fixed md:relative z-50 w-3/4 sm:w-1/3 min-w-[300px] max-w-xs md:max-w-none border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex flex-col h-full transition-transform duration-300 ease-in-out ${
            isMobile
              ? isSidebarOpen
                ? 'translate-x-0'
                : '-translate-x-full'
              : 'translate-x-0'
          }`}
        >
          <NoteList onSelectNote={handleSelectNote} />
        </div>
        <div className="flex-1 flex flex-col bg-gray-50 dark:bg-gray-900 overflow-hidden md:ml-0">
          <NoteEditor />
        </div>
      </div>
      {isProfileOpen && <Profile onClose={handleProfileClose} />}
    </div>
  );
}

function App() {
  useInitializeData();
  useInitializeAuth();

  return (
    <Router basename="/app">
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route path="/register" element={<Register />} />
        <Route path="/forgot-password" element={<ForgotPassword />} />
        <Route path="/reset-password/:token" element={<ResetPassword />} />
        <Route
          path="/"
          element={
            <ProtectedRoute>
              <AppContent />
            </ProtectedRoute>
          }
        />
      </Routes>
    </Router>
  );
}

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAuthStore();
  
  if (isLoading) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center bg-gray-50 dark:bg-gray-900">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      </div>
    );
  }
  
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }
  
  return <>{children}</>;
}

export default App;