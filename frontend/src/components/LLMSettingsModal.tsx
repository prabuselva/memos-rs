import { useState, useEffect } from 'react';
import { SettingsForm } from './SettingsForm';
import { useLLMSettings } from '../hooks/useLLMSettings';

interface LLMSettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const LLMSettingsModal = ({ isOpen, onClose }: LLMSettingsModalProps) => {
  const { loadSettings } = useLLMSettings();
  const [isOpenState, setIsOpenState] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setIsOpenState(true);
      loadSettings();
    } else {
      setIsOpenState(false);
    }
  }, [isOpen, loadSettings]);

  const handleClose = () => {
    setIsOpenState(false);
    setTimeout(() => {
      onClose();
    }, 300);
  };

  if (!isOpenState && !isOpen) return null;

  return (
    <div 
      className={`fixed inset-0 z-[60] flex items-center justify-center bg-black bg-opacity-50 backdrop-blur-sm transition-opacity duration-300 ${
        isOpenState ? 'opacity-100' : 'opacity-0'
      }`}
      onClick={handleClose}
    >
      <div 
        className={`w-full max-w-lg rounded-lg bg-white p-6 shadow-xl transition-all duration-300 dark:bg-gray-800 ${
          isOpenState ? 'scale-100 translate-y-0' : 'scale-95 translate-y-4'
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-bold text-gray-900 dark:text-white">LLM Settings</h2>
          <button 
            onClick={handleClose}
            className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          >
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <SettingsForm onCancel={handleClose} />
      </div>
    </div>
  );
};
