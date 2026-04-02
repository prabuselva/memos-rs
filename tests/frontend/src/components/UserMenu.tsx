import { useState, useEffect, useRef } from 'react';
import { useAuthStore } from '../hooks/useAuthStore';
import { LLMSettingsModal } from './LLMSettingsModal';

interface VectorDBStatus {
  enabled: boolean;
  available: boolean;
  message: string;
}

interface UserMenuProps {
  onOpenProfile?: () => void;
  onOpenLLMSettings?: () => void;
  onOpenChat?: () => void;
  vectorDBStatus?: VectorDBStatus;
}

export const UserMenu = ({ onOpenProfile, onOpenLLMSettings, onOpenChat, vectorDBStatus }: UserMenuProps) => {
  const { user, logout } = useAuthStore();
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isLLMSettingsOpen, setIsLLMSettingsOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const menuButtonRef = useRef<HTMLButtonElement>(null);
  
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        menuRef.current && 
        !menuRef.current.contains(event.target as Node) &&
        menuButtonRef.current &&
        !menuButtonRef.current.contains(event.target as Node)
      ) {
        setIsMenuOpen(false);
      }
    };
    
    document.addEventListener('mousedown', handleClickOutside);
    
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);
  
  if (!user) return null;
  
  const initials = user.username.slice(0, 2).toUpperCase();
  
  return (
    <div className="relative" ref={menuRef}>
      <button
        ref={menuButtonRef}
        onClick={(e) => {
          e.stopPropagation();
          setIsMenuOpen(!isMenuOpen);
        }}
        className="flex items-center gap-2 rounded-full bg-gray-200 px-3 py-1.5 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 transition-colors"
      >
        <div className="flex h-8 w-8 items-center justify-center rounded-full bg-blue-600 text-sm font-bold text-white shrink-0">
          {initials}
        </div>
        <span className="text-sm font-medium text-gray-700 dark:text-gray-200 hidden sm:inline">{user.username}</span>
        <svg className="h-4 w-4 text-gray-500 dark:text-gray-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      
      {isMenuOpen && (
        <div className="absolute right-0 top-full mt-2 w-48 rounded-lg bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 dark:bg-gray-800 z-50">
          <button
            onClick={(e) => {
              e.stopPropagation();
              setIsMenuOpen(false);
              if (onOpenProfile) onOpenProfile();
            }}
            className="flex w-full items-center px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-200 dark:hover:bg-gray-700"
          >
            <svg className="mr-2 h-4 w-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
            Profile
          </button>
          {vectorDBStatus?.enabled && (
            <>
              <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setIsMenuOpen(false);
                    if (onOpenChat) onOpenChat();
                  }}
                  className={`flex w-full items-center px-4 py-2 text-left text-sm transition-colors ${
                    !vectorDBStatus?.available
                      ? 'cursor-not-allowed text-gray-400 dark:text-gray-500'
                      : 'text-gray-700 hover:bg-gray-100 dark:text-gray-200 dark:hover:bg-gray-700'
                  }`}
                  disabled={!!(!vectorDBStatus?.available)}
                >
                  <svg className="mr-2 h-4 w-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
                  </svg>
                  {!vectorDBStatus?.available ? 'AI Disabled' : 'Chat'}
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setIsMenuOpen(false);
                    if (onOpenLLMSettings) {
                      setIsLLMSettingsOpen(true);
                    }
                  }}
                  className="flex w-full items-center px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100 dark:text-gray-200 dark:hover:bg-gray-700"
                >
                  <svg className="mr-2 h-4 w-4 text-gray-500 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-2.573-1.066c-1.543-.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543-.94-3.31-.826-2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                  LLM Settings
                </button>
            </>
          )}
            <button
              onClick={(e) => {
                e.stopPropagation();
                setIsMenuOpen(false);
                logout();
              }}
              className="flex w-full items-center px-4 py-2 text-left text-sm text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20"
            >
              <svg className="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
              </svg>
              Logout
            </button>
         </div>
       )}
      
      <LLMSettingsModal isOpen={isLLMSettingsOpen} onClose={() => setIsLLMSettingsOpen(false)} />
    </div>
  );
};