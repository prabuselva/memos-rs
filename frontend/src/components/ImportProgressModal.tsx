import React from 'react';

interface ImportProgressModalProps {
  isOpen: boolean;
  total: number;
  current: number;
  currentFile: string;
  status: 'idle' | 'uploading' | 'success' | 'error';
  errorMessage?: string;
}

export const ImportProgressModal: React.FC<ImportProgressModalProps> = ({
  isOpen,
  total,
  current,
  currentFile,
  status,
  errorMessage,
}) => {
  if (!isOpen) return null;

  const getProgressBarColor = () => {
    switch (status) {
      case 'uploading':
        return 'bg-blue-600';
      case 'success':
        return 'bg-green-600';
      case 'error':
        return 'bg-red-600';
      default:
        return 'bg-gray-600';
    }
  };

  const getProgressPercentage = () => {
    if (total === 0) return 0;
    return Math.round((current / total) * 100);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-lg bg-white p-6 shadow-xl dark:bg-gray-800">
        <div className="mb-4">
          <h2 className="text-xl font-bold text-gray-900 dark:text-white">
            {status === 'uploading' && 'Importing Notes...'}
            {status === 'success' && 'Import Complete!'}
            {status === 'error' && 'Import Failed'}
          </h2>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            {current} / {total} files
          </p>
        </div>

        <div className="mb-4">
          <div className="h-2 w-full overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
            <div
              className={`h-full rounded-full ${getProgressBarColor()} transition-all duration-300`}
              style={{ width: `${getProgressPercentage()}%` }}
            />
          </div>
        </div>

        <div className="mb-4 rounded bg-gray-50 p-3 dark:bg-gray-700">
          <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Processing: <span className="font-mono">{currentFile}</span>
          </p>
        </div>

        {status === 'error' && errorMessage && (
          <div className="mb-4 rounded bg-red-50 p-3 dark:bg-red-900/20">
            <p className="text-sm text-red-600 dark:text-red-400">
              {errorMessage}
            </p>
          </div>
        )}

        <div className="flex justify-end">
          {(status === 'success' || status === 'error') && (
            <button
              onClick={() => window.location.reload()}
              className="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
            >
              Close
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

ImportProgressModal.displayName = 'ImportProgressModal';