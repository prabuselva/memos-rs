import React from 'react';
import { useChatStore } from '../hooks/useChatStore';

interface ChatSidebarProps {
  isOpen: boolean;
  onClose: () => void;
}

export const ChatSidebar: React.FC<ChatSidebarProps> = ({ isOpen, onClose }) => {
  const { chatHistory, loadSession, createSession, currentSession } = useChatStore();
  
  const handleNewChat = () => {
    createSession();
    onClose();
  };
  
  return (
    <>
      {isOpen && (
        <div
          className="fixed inset-0 z-40 bg-black bg-opacity-50 backdrop-blur-sm transition-opacity"
          onClick={onClose}
        />
      )}
      <div
        className={`fixed inset-y-0 left-0 z-50 w-80 transform bg-white dark:bg-gray-900 shadow-2xl transition-transform duration-300 ease-in-out ${
          isOpen ? 'translate-x-0' : '-translate-x-full'
        }`}
      >
        <div className="flex h-full flex-col">
          <div className="flex items-center justify-between px-4 py-4 border-b border-gray-200 dark:border-gray-800">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Chat History</h2>
            <button
              onClick={onClose}
              className="rounded-lg p-2 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800"
            >
              <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
          
          <div className="flex-1 overflow-y-auto px-4 py-4">
            <button
              onClick={handleNewChat}
              className="mb-4 flex w-full items-center justify-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
            >
              <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
              </svg>
              New Chat
            </button>
            
            <div className="space-y-2">
              {chatHistory.length === 0 ? (
                <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                  <svg className="mx-auto mb-3 h-12 w-12 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
                  </svg>
                  <p>No chat history yet</p>
                  <p className="text-sm mt-1">Start a conversation to see it here</p>
                </div>
              ) : (
                chatHistory.map((session) => (
                  <button
                    key={session.id}
                    onClick={() => {
                      loadSession(session.id);
                      onClose();
                    }}
                    className={`w-full rounded-lg p-3 text-left transition-colors ${
                      currentSession?.id === session.id
                        ? 'bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800'
                        : 'hover:bg-gray-100 dark:hover:bg-gray-800'
                    }`}
                  >
                    <h3 className="font-medium text-gray-900 dark:text-white line-clamp-1">
                      {session.title}
                    </h3>
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                      {new Date(session.updatedAt).toLocaleDateString()} •{' '}
                      {session.messages.length} messages
                    </p>
                  </button>
                ))
              )}
            </div>
          </div>
          
          <div className="border-t border-gray-200 dark:border-gray-800 px-4 py-3">
            <p className="text-center text-xs text-gray-500 dark:text-gray-400">
              Chat history is stored locally in your browser
            </p>
          </div>
        </div>
      </div>
    </>
  );
};
