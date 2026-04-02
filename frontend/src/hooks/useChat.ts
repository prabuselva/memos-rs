import { useState, useCallback, useEffect } from 'react';
import { useChatStore } from '../hooks/useChatStore';
import { chatApi } from '../lib/chatApi';
import type { ChatMessageHistory } from '../lib/chatApi';

export const useChat = () => {
  const { addMessage, setError: setErrorStore, setSearchResults, selectedContextNotes, messages } = useChatStore();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setErrorInternal] = useState<string | null>(null);
  
  const setError = useCallback((msg: string | null) => {
    setErrorInternal(msg);
    setErrorStore(msg);
  }, [setErrorStore]);
  
  useEffect(() => {
    if (error) {
      setError(error);
    }
  }, [error, setError]);
  
  const search = useCallback(
    async (query: string, _mode: 'rag' | 'search' | 'chat' = 'search') => {
      if (!query.trim()) return;
      
      setIsLoading(true);
      setError(null);
      
      try {
        const response = await chatApi.search(query, _mode);
        
        const message: { id: string; role: 'assistant'; content: string; timestamp: string; searchMetadata?: any } = {
          id: Date.now().toString(),
          role: 'assistant',
          content: response.answer || `I found some relevant notes for: "${query}"`,
          timestamp: new Date().toISOString(),
        };
        
        if (response.searchMetadata) {
          message.searchMetadata = response.searchMetadata;
        }
        
        addMessage(message);
        
        if (response.results && response.results.length > 0) {
          setSearchResults(response.results);
        }
        
        return response;
      } catch (err: any) {
        console.error('Search failed:', err);
        setError(err.response?.data?.message || 'Failed to search notes');
        addMessage({
          id: Date.now().toString(),
          role: 'assistant',
          content: `I encountered an error while searching: ${err.response?.data?.message || 'Please try again'}`,
          timestamp: new Date().toISOString(),
        });
      } finally {
        setIsLoading(false);
      }
    },
    [addMessage, setError, setSearchResults]
  );
  
  const chat = useCallback(
    async (query: string, contextNotes?: string[], mode: 'rag' | 'search' | 'chat' = 'rag') => {
      if (!query.trim()) return;
      
      setIsLoading(true);
      setError(null);
      
      const history: ChatMessageHistory[] = messages.map(m => ({
          role: m.role,
          content: m.content,
          timestamp: m.timestamp,
        }));
      
      try {
        const response = await chatApi.chat(query, contextNotes || selectedContextNotes, mode, history);
        
        const message: { id: string; role: 'assistant'; content: string; timestamp: string; sources?: any[]; references?: any[]; searchMetadata?: any } = {
          id: Date.now().toString(),
          role: 'assistant' as const,
          content: response.answer,
          timestamp: new Date().toISOString(),
        };
        
        if (response.sources && response.sources.length > 0) {
          message.sources = response.sources;
        }
        
        if (response.references && response.references.length > 0) {
          message.references = response.references;
        }
        
        if (response.searchMetadata) {
          message.searchMetadata = response.searchMetadata;
        }
        
        addMessage(message);
        
        return response;
      } catch (err: any) {
        console.error('Chat failed:', err);
        const errorMsg = err.response?.data?.message || 'Failed to get response';
        setError(errorMsg);
        
        addMessage({
          id: Date.now().toString(),
          role: 'assistant',
          content: `I encountered an error: ${errorMsg}`,
          timestamp: new Date().toISOString(),
        });
      } finally {
        setIsLoading(false);
      }
    },
    [addMessage, setError, selectedContextNotes, messages]
  );
  
  const loadNotes = useCallback(
    async (noteIds: string[]) => {
      try {
        const notes = await chatApi.getNotesByIds(noteIds);
        return notes;
      } catch (err) {
        console.error('Failed to load notes:', err);
        return [];
      }
    },
    []
  );
  
  return {
    isLoading,
    error,
    search,
    chat,
    loadNotes,
  };
};
