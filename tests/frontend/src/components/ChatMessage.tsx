import React from 'react';
import type { ChatMessage as ChatMessageType } from '../hooks/useChatStore';
import { useNoteStore } from '../hooks/useNoteStore';
import { MarkdownPreview } from './MarkdownPreview';

interface ChatMessageProps {
  message: ChatMessageType;
}

const ChatMessage: React.FC<ChatMessageProps> = ({ message }) => {
  const isUser = message.role === 'user';
  const { notes } = useNoteStore();
  
  const getNoteTitle = (noteId: string | number) => {
    const note = notes.find((n) => n.id === noteId.toString());
    return note?.title || 'Untitled Note';
  };
  
  const renderReferences = (references?: any[]) => {
    if (!references || references.length === 0) return null;
    
    return (
      <div className="mt-4 pt-3 border-t border-gray-200 dark:border-gray-700">
        <p className="text-xs text-gray-500 dark:text-gray-400 mb-2">
          References ({references.length})
        </p>
        <div className="space-y-2">
          {references.map((ref: any) => (
            <div key={ref.id} className="p-2 rounded bg-gray-50 dark:bg-gray-700/50">
              <div className="flex items-center justify-between">
                <span className="text-xs font-medium text-gray-900 dark:text-gray-200">
                  {ref.title || `Note ${ref.note_id?.slice(0, 8)}`}
                </span>
                <span className="text-xs text-gray-500">
                  Score: {ref.score.toFixed(2)}
                </span>
              </div>
              {ref.content_snippet && (
                <p className="text-xs text-gray-600 dark:text-gray-400 mt-1 line-clamp-2">
                  {ref.content_snippet}
                </p>
              )}
            </div>
          ))}
        </div>
      </div>
    );
  };
  
  const renderSearchMetadata = (metadata?: any) => {
    if (!metadata) return null;
    
    return (
      <div className="mt-2 text-xs text-gray-500 dark:text-gray-400 flex flex-wrap gap-3">
        <span>Search: {metadata.vector_search_time_ms}ms</span>
        <span>Generation: {metadata.llm_generation_time_ms}ms</span>
        <span>Tokens: {metadata.total_tokens}</span>
        {metadata.filtered_count > 0 && (
          <span>Filtered: {metadata.filtered_count}</span>
        )}
      </div>
    );
  };
  
  return (
    <div
      className={`flex w-full ${isUser ? 'justify-end' : 'justify-start'} mb-6 animate-in fade-in slide-in-from-bottom-2 duration-300`}
    >
      <div
        className={`max-w-[85%] md:max-w-[75%] rounded-2xl px-5 py-4 shadow-md ${
          isUser
            ? 'bg-blue-600 text-white rounded-br-none'
            : 'bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 rounded-bl-none'
        }`}
      >
        <div className="prose dark:prose-invert max-w-none">
          {message.role === 'user' ? (
            <p className="whitespace-pre-wrap leading-relaxed">{message.content}</p>
          ) : (
            <MarkdownPreview content={message.content} />
          )}
        </div>
        
        {message.sources && message.sources.length > 0 && (
          <div className={`mt-4 pt-3 border-t ${isUser ? 'border-blue-400/30' : 'border-gray-200 dark:border-gray-700'}`}>
            <p className={`text-xs ${isUser ? 'text-blue-100' : 'text-gray-500 dark:text-gray-400'} mb-2`}>
              Sources:
            </p>
            <div className="flex flex-wrap gap-2">
              {message.sources.map((source: any, index) => (
                <span
                  key={index}
                  className={`rounded px-2 py-1 text-xs ${
                    isUser
                      ? 'bg-blue-500/30 text-blue-100'
                      : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'
                  }`}
                  title={source.title}
                >
                  {source.title || getNoteTitle(source.id)}
                </span>
              ))}
            </div>
          </div>
        )}
        
        {renderReferences(message.references)}
        {renderSearchMetadata(message.searchMetadata)}
        
        <div
          className={`mt-2 text-xs ${
            isUser ? 'text-blue-200/70' : 'text-gray-400 dark:text-gray-500'
          }`}
        >
          {new Date(message.timestamp).toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit',
          })}
        </div>
      </div>
    </div>
  );
};

export { ChatMessage };
