import { useState, useRef, useEffect } from 'react';
import { useChatStore, type ChatMode } from '../hooks/useChatStore';
import { useChat } from '../hooks/useChat';
import { ChatMessage } from './ChatMessage';
import { ChatSidebar } from './ChatSidebar';

interface VectorDBStatus {
  enabled: boolean;
  available: boolean;
  message: string;
}

interface ChatInterfaceProps {
  isOpen: boolean;
  onClose: () => void;
  vectorDBStatus?: VectorDBStatus;
}

export const ChatInterface: React.FC<ChatInterfaceProps> = ({ isOpen, onClose, vectorDBStatus }) => {
   const {
      messages,
      currentSession,
      currentMode,
      setSearchResults,
      searchResults,
      addMessage,
      setError,
      createSession,
      removeContextNote,
      selectedContextNotes,
      setMode,
      clearCurrentSession,
    } = useChatStore();
  
  const { isLoading, error, chat, search } = useChat();
  const [inputValue, setInputValue] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  if (isOpen && vectorDBStatus && !vectorDBStatus.available) {
    return (
      <div
        className={`fixed inset-0 z-50 flex flex-col items-center justify-center bg-gray-50 dark:bg-gray-900 transition-opacity duration-300 ${
          isOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'
        }`}
      >
        <div className="w-full max-w-lg rounded-lg bg-white p-8 shadow-2xl dark:bg-gray-800">
          <div className="flex flex-col items-center text-center">
            <div className="mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-red-100 dark:bg-red-900">
              <svg className="h-10 w-10 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
            </div>
            <h2 className="mb-4 text-2xl font-bold text-gray-900 dark:text-white">AI Features Disabled</h2>
            <p className="mb-6 text-gray-600 dark:text-gray-300">
              {vectorDBStatus.message}
            </p>
            <div className="rounded-lg bg-yellow-50 dark:bg-yellow-900/20 p-4 max-w-md">
              <p className="text-sm text-yellow-800 dark:text-yellow-300">
                <strong>To enable AI features:</strong>
                <ol className="mt-2 ml-4 list-decimal space-y-1 text-xs">
                  <li>Configure Qdrant or another vector database</li>
                  <li>Update the configuration file with vector database settings</li>
                  <li>Restart the backend server</li>
                </ol>
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }
  const inputRef = useRef<HTMLTextAreaElement>(null);
  
  useEffect(() => {
    if (isOpen && currentSession === null) {
      createSession();
    }
  }, [isOpen, currentSession, createSession]);
  
  useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 100);
    }
  }, [isOpen, currentSession]);
  
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);
  
  useEffect(() => {
    if (error) {
      setError(error);
    }
  }, [error, setError]);
  
  const handleSend = async () => {
     if (!inputValue.trim() || !currentSession) return;
     
     const userMessage = {
       id: Date.now().toString(),
       role: 'user' as const,
       content: inputValue.trim(),
       timestamp: new Date().toISOString(),
     };
     
     addMessage(userMessage);
       setInputValue('');
         
         if (currentMode === 'search') {
            await search(inputValue.trim());
          } else {
            await chat(inputValue.trim(), selectedContextNotes, currentMode);
          }
       };
   
   const handleKeyDown = (e: React.KeyboardEvent) => {
     if (e.key === 'Enter' && !e.shiftKey) {
       e.preventDefault();
       handleSend();
     }
   };
   
 const handleModeChange = (mode: ChatMode) => {
       if (currentMode !== mode) {
         setMode(mode);
         setSearchResults([]);
       }
     };
   
   return (
    <>
      <ChatSidebar isOpen={isOpen} onClose={onClose} />
      
      <div
        className={`fixed inset-0 z-50 flex flex-col bg-gray-50 dark:bg-gray-900 transition-opacity duration-300 ${
          isOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'
        }`}
      >
        <div className="flex-1 flex flex-col max-w-5xl mx-auto w-full h-full bg-white dark:bg-gray-800 shadow-2xl rounded-lg m-4 overflow-hidden">
          {/* Header */}
          <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
            <div className="flex items-center gap-4">
              <button
                onClick={onClose}
                className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                aria-label="Close chat"
              >
                <svg className="h-6 w-6 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
              
              <h2 className="text-xl font-semibold text-gray-900 dark:text-white flex items-center gap-2">
                <svg className="h-6 w-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
                </svg>
                AI Chat
              </h2>
            </div>
            
            <div className="flex items-center gap-2">
              <div className="flex items-center bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
                <button
                  onClick={() => handleModeChange('rag')}
                  className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors ${
                    currentMode === 'rag'
                      ? 'bg-white dark:bg-gray-600 text-blue-600 dark:text-blue-400 shadow-sm'
                      : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                  }`}
                >
                  RAG
                </button>
                <button
                  onClick={() => handleModeChange('search')}
                  className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors ${
                    currentMode === 'search'
                      ? 'bg-white dark:bg-gray-600 text-blue-600 dark:text-blue-400 shadow-sm'
                      : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                  }`}
                >
                  Search
                </button>
                <button
                  onClick={() => handleModeChange('chat')}
                  className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors ${
                    currentMode === 'chat'
                      ? 'bg-white dark:bg-gray-600 text-blue-600 dark:text-blue-400 shadow-sm'
                      : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                  }`}
                >
                  Chat
                </button>
              </div>
            </div>
          </div>
          
          {/* Context Notes (only in chat mode) */}
          {currentMode === 'chat' && selectedContextNotes.length > 0 && (
            <div className="px-6 py-3 bg-blue-50 dark:bg-blue-900/20 border-b border-blue-100 dark:border-blue-800">
              <p className="text-xs font-medium text-blue-800 dark:text-blue-300 mb-2">Context Notes:</p>
              <div className="flex flex-wrap gap-2">
                {selectedContextNotes.map((noteId) => (
                  <span
                    key={noteId}
                    className="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-blue-100 dark:bg-blue-800 text-blue-800 dark:text-blue-200 text-xs"
                  >
                    Note #{noteId.slice(0, 8)}
                    <button
                      onClick={() => removeContextNote(noteId)}
                      className="hover:text-blue-600 dark:hover:text-blue-100"
                    >
                      <svg className="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </span>
                ))}
              </div>
            </div>
          )}
          
          {/* Messages */}
          <div className="flex-1 overflow-y-auto p-6 space-y-6">
            {messages.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-full text-center px-4">
                <div className="w-20 h-20 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center mb-6">
                  <svg className="h-10 w-10 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
                  </svg>
                </div>
                <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                  Start a Conversation
                </h3>
                <p className="text-gray-500 dark:text-gray-400 max-w-md">
                  {currentMode === 'search'
                    ? 'Search your notes using natural language. I will find the most relevant results for you.'
                    : 'Chat with me about your notes. I can provide context-aware responses using your note content.'}
                </p>
                
                {currentMode === 'search' && (
                  <div className="mt-8 grid grid-cols-1 md:grid-cols-2 gap-4 max-w-2xl w-full">
                    <div className="p-4 rounded-lg bg-white dark:bg-gray-700 shadow-sm">
                      <p className="text-sm font-medium text-gray-900 dark:text-white mb-2">Find notes about:</p>
                      <ul className="space-y-2 text-sm text-gray-600 dark:text-gray-300">
                        <li className="flex items-center gap-2">
                          <span className="text-blue-500">✓</span> Project planning
                        </li>
                        <li className="flex items-center gap-2">
                          <span className="text-blue-500">✓</span> Technical documentation
                        </li>
                        <li className="flex items-center gap-2">
                          <span className="text-blue-500">✓</span> Meeting notes
                        </li>
                      </ul>
                    </div>
                    <div className="p-4 rounded-lg bg-white dark:bg-gray-700 shadow-sm">
                      <p className="text-sm font-medium text-gray-900 dark:text-white mb-2">Try asking:</p>
                      <ul className="space-y-2 text-sm text-gray-600 dark:text-gray-300">
                        <li className="flex items-center gap-2">
                          <span className="text-blue-500">✓</span> "Show me notes about React"
                        </li>
                        <li className="flex items-center gap-2">
                          <span className="text-blue-500">✓</span> "Find my project plans"
                        </li>
                      </ul>
                    </div>
                  </div>
                )}
              </div>
            ) : (
              <>
                {messages.map((message) => (
                  <ChatMessage key={message.id} message={message} />
                ))}
                
                {isLoading && (
                  <div className="flex justify-start animate-pulse">
                    <div className="bg-gray-100 dark:bg-gray-700 rounded-2xl px-5 py-4 rounded-bl-none">
                      <div className="flex items-center gap-2">
                        <div className="h-2 w-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
                        <div className="h-2 w-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
                        <div className="h-2 w-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
                      </div>
                    </div>
                  </div>
                )}
                
                <div ref={messagesEndRef} />
              </>
            )}
          </div>
          
          {/* Search Results (only in search mode) */}
           {currentMode === 'search' && (
             <div className="flex-1 overflow-y-auto px-6 pb-4">
               <div className="flex items-center gap-2 text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                 <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                   <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                 </svg>
                 <span>Search Results</span>
               </div>
               <div className="space-y-3">
                 {searchResults.length === 0 ? (
                   <div className="text-center py-8 text-gray-500 dark:text-gray-400">
                     <p>Start typing to search your notes...</p>
                   </div>
                 ) : (
                   searchResults.map((note) => (
                     <div key={note.id} className="p-4 rounded-lg bg-gray-50 dark:bg-gray-700/50 hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-colors cursor-pointer border border-gray-200 dark:border-gray-600">
                       <h4 className="font-medium text-gray-900 dark:text-white mb-1">{note.title}</h4>
                       <p className="text-sm text-gray-600 dark:text-gray-300 line-clamp-2">{note.content}</p>
                     </div>
                   ))
                 )}
               </div>
             </div>
           )}
          
          {/* Input Area */}
           <div className="border-t border-gray-200 dark:border-gray-700 p-4 bg-white dark:bg-gray-800">
             <div className="flex items-end gap-3">
               <div className="flex-1 relative">
                 <textarea
                   ref={inputRef}
                   value={inputValue}
                   onChange={(e) => setInputValue(e.target.value)}
                   onKeyDown={handleKeyDown}
                   placeholder={
                     currentMode === 'search'
                       ? 'Search your notes...'
                       : 'Type your message...'
                   }
                   className="w-full max-h-32 min-h-[44px] rounded-lg border border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 px-4 py-3 pr-20 text-gray-900 dark:text-gray-100 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 resize-none text-sm"
                   rows={1}
                   style={{ minHeight: '44px' }}
                 />
                 <div className="absolute top-3 right-3 flex items-center gap-1">
                   <button
                     onClick={() => setInputValue('')}
                     className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                   >
                     <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                       <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                     </svg>
                   </button>
                   <button
                     onClick={clearCurrentSession}
                     disabled={messages.length === 0 || isLoading}
                     className="text-gray-400 hover:text-red-600 dark:hover:text-red-400 disabled:opacity-30 disabled:cursor-not-allowed"
                     title="Clear session"
                   >
                     <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                       <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                     </svg>
                   </button>
                 </div>
               </div>
               <button
                 onClick={handleSend}
                 disabled={!inputValue.trim() || isLoading}
                 className="flex items-center justify-center w-10 h-10 rounded-lg bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
               >
                 <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                   <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
                 </svg>
               </button>
             </div>
             <div className="mt-2 flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
               <span>Press Enter to send, Shift+Enter for new line</span>
               {isLoading && <span>Thinking...</span>}
             </div>
           </div>
        </div>
      </div>
    </>
  );
};
