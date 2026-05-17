import { create } from 'zustand';
import type { InstalledSkill } from '../types/skill';
import {
  getInstalledSkills,
  installSkill as apiInstall,
  uninstallSkill as apiUninstall,
  toggleSkillEnabled as apiToggle,
  batchAssess,
} from '../services/tauri';

interface InstalledState {
  items: InstalledSkill[];
  isLoading: boolean;
  error: string | null;
  isEmpty: boolean;
  fetchItems: () => Promise<void>;
  install: (packageId: string) => Promise<void>;
  uninstall: (skillId: string) => Promise<void>;
  toggle: (skillId: string, enabled: boolean) => Promise<void>;
}

const useInstalledStore = create<InstalledState>((set, get) => ({
  items: [],
  isLoading: false,
  error: null,
  get isEmpty() {
    return get().items.length === 0 && !get().isLoading;
  },

  fetchItems: async () => {
    set({ isLoading: true, error: null });
    try {
      const items = await getInstalledSkills();
      set({ items, isLoading: false });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to fetch installed skills';
      set({ error: message, isLoading: false });
    }
  },

  install: async (packageId: string) => {
    set({ isLoading: true, error: null });
    try {
      const skill = await apiInstall(packageId);
      set((state) => ({
        items: [...state.items, skill],
        isLoading: false,
      }));
      // Run assessments on all skills after a new install
      batchAssess().catch(() => {});
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to install skill';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  uninstall: async (skillId: string) => {
    set({ isLoading: true, error: null });
    try {
      await apiUninstall(skillId);
      set((state) => ({
        items: state.items.filter((s) => s.id !== skillId),
        isLoading: false,
      }));
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to uninstall skill';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  toggle: async (skillId: string, enabled: boolean) => {
    try {
      await apiToggle(skillId, enabled);
      set((state) => ({
        items: state.items.map((s) =>
          s.id === skillId ? { ...s, enabled } : s,
        ),
      }));
    } catch (err) {
      const message =
        err instanceof Error
          ? err.message
          : 'Failed to toggle skill enabled state';
      set({ error: message });
      throw err;
    }
  },
}));

export { useInstalledStore };
