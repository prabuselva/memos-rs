import { useEffect, useCallback, useState } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { useNoteStore } from './hooks/useNoteStore';
import { useInitializeData } from './hooks/useData';
import { useInitializeAuth } from './hooks/useAuthService';
import { useAuthStore } from './hooks/useAuthStore';
import { NotebookSidebar, NotesSidebar, SearchSidebarContent, NoteEditor, Header, Profile, ChatInterface, LLMSettingsModal, VectorDBStatusModal } from './components';
import { Login } from './components/Login';
import { Register } from './components/Register';
import { ForgotPassword } from './components/ForgotPassword';
import { ResetPassword } from './components/ResetPassword';
import { notesApi, vectorDbApi, type VectorDBStatus, type Notebook } from './lib/api';
import { getVersion } from './lib/version';

function AppContent() {
    const { isDarkMode, addNote, selectNote, setError, selectedNotebook, setSelectedNotebook, setSearchQuery } = useNoteStore();
    const [isMobile, setIsMobile] = useState(false);
    const [isSidebarOpen, setIsSidebarOpen] = useState(false);
   const [isProfileOpen, setIsProfileOpen] = useState(false);
    const [isChatOpen, setIsChatOpen] = useState(false);
    const [isLLMSettingsOpen, setIsLLMSettingsOpen] = useState(false);
    const [vectorDBStatus, setVectorDBStatus] = useState<VectorDBStatus | null>(null);
    const [isVectorDBModalOpen, setIsVectorDBModalOpen] = useState(false);
    const [selectedNotebookForNotes, setSelectedNotebookForNotes] = useState<Notebook | null | undefined>(undefined);
     const { searchQuery } = useNoteStore();
   
   const openProfile = () => {
      setIsProfileOpen(true);
      setError(null);
    };

  const openChat = useCallback(() => {
    if (vectorDBStatus && !vectorDBStatus.available) {
      setIsVectorDBModalOpen(true);
    } else {
      setIsChatOpen(true);
    }
  }, [vectorDBStatus]);
  
  const closeChat = useCallback(() => {
    setIsChatOpen(false);
  }, []);
  
  const handleProfileClose = () => {
       setIsProfileOpen(false);
     };
   
  const handleOpenNotes = (notebook: Notebook | null) => {
      if (notebook) {
        setSelectedNotebook(notebook.id);
      } else {
        setSelectedNotebook(null);
      }
      setSelectedNotebookForNotes(notebook);
    };
   
   const handleCloseNotes = () => {
      setSelectedNotebookForNotes(undefined);
    };
    
   const handleLLMSettingsClose = () => {
  setIsLLMSettingsOpen(false);
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

 useEffect(() => {
    const checkVectorDBStatus = async () => {
      try {
        const status = await vectorDbApi.getStatus();
        setVectorDBStatus(status);
      } catch (err) {
        console.error('Failed to check Vector DB status:', err);
      }
    };
    checkVectorDBStatus();
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'k') {
        e.preventDefault();
        openChat();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [openChat]);

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
    } catch {
      setError('Failed to create note');
    }
  }, [addNote, selectNote, setError]);

  return (
    <div className={`flex h-screen flex-col ${isDarkMode ? 'dark' : ''}`}>
      <Header onNewNote={handleNewNote} onMenuToggle={() => setIsSidebarOpen(!isSidebarOpen)} onOpenProfile={openProfile} onOpenLLMSettings={() => setIsLLMSettingsOpen(true)} onOpenChat={openChat} />
      <div className="flex-1 flex overflow-hidden bg-gray-50 dark:bg-gray-900 relative">
        {isMobile && (
          <div
            className={`fixed inset-0 bg-black bg-opacity-50 z-40 transition-opacity duration-300 ${
              isSidebarOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'
            }`}
            onClick={() => setIsSidebarOpen(false)}
          />
        )}
          {isSidebarOpen && (
          <div className="relative z-50 w-[300px] border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex flex-col h-full overflow-hidden transition-transform duration-300 ease-in-out">
            <NotebookSidebar 
              selectedNotebookId={selectedNotebook} 
              onOpenNotes={handleOpenNotes}
              onNoteSelect={(notebookId) => {
                if (notebookId) {
                  setSelectedNotebook(notebookId);
                } else {
                  setSelectedNotebook(null);
                }
              }}
            />
            {selectedNotebookForNotes !== undefined && (
              <NotesSidebar 
                notebook={selectedNotebookForNotes}
                onClose={handleCloseNotes}
                onSelectNote={(noteId) => {
                  selectNote(noteId);
                }}
              />
            )}
          </div>
        )}
        {searchQuery && (
          <div className="absolute inset-0 z-[60] bg-black bg-opacity-50 flex justify-start" onClick={() => setSearchQuery('')}>
            <div
              className="h-full w-[300px] bg-white dark:bg-gray-800 shadow-2xl flex flex-col"
              onClick={(e) => e.stopPropagation()}
              onKeyDown={(e) => {
                if (e.key === 'Escape') {
                  setSearchQuery('');
                }
              }}
              tabIndex={0}
            >
              <div className="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
                <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                  Search: {searchQuery}
                </h2>
                <button
                  onClick={() => setSearchQuery('')}
                  className="p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                >
                  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
              <SearchSidebarContent query={searchQuery} onSelectNote={selectNote} />
            </div>
          </div>
        )}
        <div className="flex-1 flex flex-col bg-gray-50 dark:bg-gray-900 overflow-hidden">
          <NoteEditor />
        </div>
      </div>
         {isProfileOpen && <Profile onClose={handleProfileClose} />}
           <ChatInterface isOpen={isChatOpen} onClose={closeChat} vectorDBStatus={vectorDBStatus ?? undefined} />
           {isLLMSettingsOpen && <LLMSettingsModal isOpen={isLLMSettingsOpen} onClose={handleLLMSettingsClose} />}
         <footer className="px-4 py-2 border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex-shrink-0 text-center text-xs text-gray-500 dark:text-gray-400">
           Memos RS v{getVersion()}
         </footer>
         <VectorDBStatusModal
           isOpen={isVectorDBModalOpen}
           onClose={() => setIsVectorDBModalOpen(false)}
           status={vectorDBStatus || { enabled: false, available: false, message: 'Vector DB status not available' }}
         />
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
