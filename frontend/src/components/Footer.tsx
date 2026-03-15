import { APP_VERSION } from '../lib/version';

export const Footer = () => {
  return (
    <footer className="px-4 py-2 border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
      <span>Memos RS</span>
      <span>v{APP_VERSION}</span>
    </footer>
  );
};