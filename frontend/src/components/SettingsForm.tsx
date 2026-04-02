import { useState, useEffect } from 'react';
import { useLLMSettings } from '../hooks/useLLMSettings';

interface SettingsFormProps {
  onSave?: () => void;
  onCancel?: () => void;
}

export const SettingsForm = ({ onSave, onCancel }: SettingsFormProps) => {
  const { settings, setSettings, saveSettings, testConnection, error, isLoading } = useLLMSettings();
  const [isTesting, setIsTesting] = useState(false);
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null);

  useEffect(() => {
    if (error) {
      setTestResult('error');
    }
  }, [error]);

  const handleChange = (field: keyof typeof settings, value: any) => {
    setSettings({ [field]: value });
    if (testResult) {
      setTestResult(null);
    }
  };

  const handleTestConnection = async () => {
    setIsTesting(true);
    setTestResult(null);
    
    const success = await testConnection();
    
    setTestResult(success ? 'success' : 'error');
    setIsTesting(false);
  };

 const handleSave = async () => {
     await saveSettings({});
     if (onSave) onSave();
   };

  const providers = ['openai', 'ollama', 'anthropic', 'groq'];

  return (
    <div className="space-y-6">
      {testResult === 'success' && (
        <div className="rounded-lg bg-green-100 p-4 text-green-800 dark:bg-green-900 dark:text-green-200">
          Connection successful!
        </div>
      )}
      {testResult === 'error' && (
        <div className="rounded-lg bg-red-100 p-4 text-red-800 dark:bg-red-900 dark:text-red-200">
          Connection failed. Please check your settings.
        </div>
      )}

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Provider
        </label>
        <select
          value={settings.provider}
          onChange={(e) => handleChange('provider', e.target.value)}
          disabled={isLoading}
          className="w-full rounded-lg border border-gray-300 bg-white px-4 py-2 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
        >
          {providers.map((provider) => (
            <option key={provider} value={provider}>
              {provider.charAt(0).toUpperCase() + provider.slice(1)}
            </option>
          ))}
        </select>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          API URL
        </label>
       <input
           type="text"
           value={settings.url}
           onChange={(e) => handleChange('url', e.target.value)}
           disabled={isLoading}
           className="w-full rounded-lg border border-gray-300 bg-white px-4 py-2 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
           placeholder="http://localhost:11434/v1"
         />
         <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
           The base URL for your LLM provider API (e.g., http://localhost:11434/v1 for llama.cpp)
         </p>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          API Key (optional)
        </label>
      <input
           type="password"
           value={settings.api_key || ''}
           onChange={(e) => handleChange('api_key', e.target.value)}
           disabled={isLoading}
           className="w-full rounded-lg border border-gray-300 bg-white px-4 py-2 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
           placeholder={settings.provider === 'openai' ? 'Enter your API key (optional for llama.cpp)' : 'Enter your API key'}
         />
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Model Name
        </label>
        <input
          type="text"
          value={settings.model}
          onChange={(e) => handleChange('model', e.target.value)}
          disabled={isLoading}
          className="w-full rounded-lg border border-gray-300 bg-white px-4 py-2 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
          placeholder="llama3"
        />
        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
          The model name to use (e.g., llama3, gpt-3.5-turbo)
        </p>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Temperature: {settings.temperature}
        </label>
        <input
          type="range"
          min="0"
          max="1"
          step="0.1"
          value={settings.temperature}
          onChange={(e) => handleChange('temperature', parseFloat(e.target.value))}
          disabled={isLoading}
          className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
        />
        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
          Controls randomness: 0 = deterministic, 1 = creative
        </p>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Max Tokens
        </label>
        <input
          type="number"
          value={settings.max_tokens}
          onChange={(e) => handleChange('max_tokens', parseInt(e.target.value))}
          disabled={isLoading}
          min="1"
          max="4096"
          className="w-full rounded-lg border border-gray-300 bg-white px-4 py-2 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
        />
        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
          Maximum number of tokens in the response
        </p>
      </div>

      <div className="flex gap-4 pt-4">
        <button
          onClick={handleTestConnection}
          disabled={isLoading || isTesting}
          className="flex-1 rounded-lg bg-gray-600 px-4 py-2 font-medium text-white hover:bg-gray-700 disabled:opacity-50"
        >
          {isTesting ? 'Testing...' : 'Test Connection'}
        </button>
      </div>

      <div className="flex gap-4 pt-4 border-t border-gray-200 dark:border-gray-700">
        <button
          onClick={onCancel}
          disabled={isLoading}
          className="flex-1 rounded-lg bg-gray-500 px-4 py-2 font-medium text-white hover:bg-gray-600 disabled:opacity-50"
        >
          Cancel
        </button>
        <button
          onClick={handleSave}
          disabled={isLoading}
          className="flex-1 rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700 disabled:opacity-50"
        >
          {isLoading ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    </div>
  );
};
