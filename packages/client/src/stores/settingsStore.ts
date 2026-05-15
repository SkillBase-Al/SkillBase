import { create } from 'zustand';
import type { AppSettings } from '../types/settings';
import { getSettings, updateSettings as apiUpdateSettings } from '../services/tauri';

interface SettingsState {
  settings: AppSettings | null;
  isLoading: boolean;
  error: string | null;
  fetchSettings: () => Promise<void>;
  updateSettings: (settings: Partial<AppSettings>) => Promise<void>;
}

const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: null,
  isLoading: false,
  error: null,

  fetchSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      const settings = await getSettings();
      set({ settings, isLoading: false });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to fetch settings';
      set({ error: message, isLoading: false });
    }
  },

  updateSettings: async (partial: Partial<AppSettings>) => {
    set({ isLoading: true, error: null });
    try {
      const current = get().settings ?? await getSettings();
      const merged: AppSettings = { ...current, ...partial };
      const settings = await apiUpdateSettings(merged);
      set({ settings, isLoading: false });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to update settings';
      set({ error: message, isLoading: false });
      throw err;
    }
  },
}));

export { useSettingsStore };
