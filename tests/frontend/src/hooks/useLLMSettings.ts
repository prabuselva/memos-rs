import { create } from 'zustand';
import { authApi } from '../lib/api';

interface LLMSettingsState {
  settings: {
    provider: string;
    url: string;
    api_key: string | null;
    model: string;
    temperature: number;
    max_tokens: number;
  };
  isLoading: boolean;
  error: string | null;
  
  loadSettings: () => Promise<void>;
  saveSettings: (settings: Partial<LLMSettingsState['settings']>) => Promise<void>;
  testConnection: () => Promise<boolean>;
  setSettings: (settings: Partial<LLMSettingsState['settings']>) => void;
}

const DEFAULT_SETTINGS = {
  provider: 'openai',
  url: 'http://localhost:11434/v1',
  api_key: null,
  model: 'llama3',
  temperature: 0.7,
  max_tokens: 2048,
};

export const useLLMSettings = create<LLMSettingsState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  isLoading: false,
  error: null,
  
  loadSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await authApi.getLLMSettings();
      if (response.data) {
        set({ 
          settings: response.data,
          isLoading: false,
          error: null,
        });
      }
    } catch (error: any) {
      console.error('Failed to load LLM settings:', error);
      set({ 
        isLoading: false,
        error: error.response?.data?.message || 'Failed to load settings',
      });
    }
  },
  
  saveSettings: async (newSettings) => {
    set({ isLoading: true, error: null });
    try {
      const currentSettings = get().settings;
      const settingsToUpdate = { ...currentSettings, ...newSettings };
      
      const response = await authApi.updateLLMSettings(settingsToUpdate);
      if (response.data) {
        set({ 
          settings: response.data,
          isLoading: false,
          error: null,
        });
      }
    } catch (error: any) {
      console.error('Failed to save LLM settings:', error);
      set({ 
        isLoading: false,
        error: error.response?.data?.message || 'Failed to save settings',
      });
    }
  },
  
  testConnection: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await authApi.testLLMConnection(get().settings);
      set({ isLoading: false });
      return response.status === 200;
    } catch (error: any) {
      console.error('Connection test failed:', error);
      set({ 
        isLoading: false,
        error: error.response?.data?.message || 'Connection test failed',
      });
      return false;
    }
  },
  
  setSettings: (settings) => set({ settings: { ...get().settings, ...settings } }),
}));
