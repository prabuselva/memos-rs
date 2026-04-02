import React from 'react';

interface VectorDBStatusModalProps {
  isOpen: boolean;
  onClose: () => void;
  status: {
    enabled: boolean;
    available: boolean;
    message: string;
  };
}

export const VectorDBStatusModal: React.FC<VectorDBStatusModalProps> = ({ 
  isOpen, 
  onClose, 
  status 
}) => {
  if (!isOpen) return null;

  const { available, message } = status;

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm p-4">
      <div className="w-full max-w-md rounded-lg bg-white p-6 shadow-xl dark:bg-gray-800">
        <div className="flex items-center gap-3 mb-4">
          {available ? (
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-green-100 dark:bg-green-900">
              <svg className="h-6 w-6 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
          ) : (
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-red-100 dark:bg-red-900">
              <svg className="h-6 w-6 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </div>
          )}
          <h2 className="text-xl font-bold text-gray-900 dark:text-white">
            {available ? 'AI Features Enabled' : 'AI Features Disabled'}
          </h2>
        </div>
        
        <div className="mb-6">
          <p className="text-gray-700 dark:text-gray-300 mb-2">
            {message}
          </p>
          
          {!available && (
            <div className="rounded-lg bg-yellow-50 dark:bg-yellow-900/20 p-4">
              <p className="text-sm text-yellow-800 dark:text-yellow-300">
                <strong>Note:</strong> To enable AI features, you need to:
                <ol className="mt-2 ml-4 list-decimal space-y-1 text-xs">
                  <li>Configure Qdrant or another vector database</li>
                  <li>Update the configuration file with vector database settings</li>
                  <li>Restart the backend server</li>
                </ol>
              </p>
            </div>
          )}
        </div>
        
        <div className="flex justify-end">
          <button
            onClick={onClose}
            className="rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700 dark:bg-blue-700 dark:hover:bg-blue-800"
          >
            OK
          </button>
        </div>
      </div>
    </div>
  );
};
