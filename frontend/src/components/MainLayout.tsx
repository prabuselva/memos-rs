import { useNoteStore } from '../hooks/useNoteStore';
import { NoteEditor } from './NoteEditor';

export const MainLayout = () => {
  const { isDarkMode } = useNoteStore();

  return (
    <div className={`flex h-screen flex-col ${isDarkMode ? 'dark' : ''}`}>
      <div className="flex-1 flex overflow-hidden bg-gray-50 dark:bg-gray-900">
        <div className="w-1/3 min-w-[300px] border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex flex-col">
          <div className="p-4 border-b border-gray-200 dark:border-gray-700">
            <h1 className="text-xl font-bold text-blue-600 dark:text-blue-400">Memos RS</h1>
          </div>
          <NoteEditor />
        </div>
      </div>
    </div>
  );
};