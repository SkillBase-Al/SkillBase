import { create } from 'zustand';
import type { DedupGroup } from '../types/dedup';
import {
  runDedup as apiRunDedup,
  getDedupGroups,
  deleteSkillFromGroup as apiDeleteSkill,
} from '../services/tauri';

interface DedupState {
  groups: DedupGroup[];
  isLoading: boolean;
  error: string | null;
  isEmpty: boolean;
  runDedup: () => Promise<void>;
  fetchGroups: () => Promise<void>;
  deleteSkill: (groupId: string, skillId: string) => Promise<void>;
}

const useDedupStore = create<DedupState>((set, get) => ({
  groups: [],
  isLoading: false,
  error: null,
  get isEmpty() {
    return get().groups.length === 0 && !get().isLoading;
  },

  runDedup: async () => {
    set({ isLoading: true, error: null });
    try {
      const groups = await apiRunDedup();
      set({ groups, isLoading: false });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to run dedup';
      set({ error: message, isLoading: false });
    }
  },

  fetchGroups: async () => {
    set({ isLoading: true, error: null });
    try {
      const groups = await getDedupGroups();
      set({ groups, isLoading: false });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to fetch dedup groups';
      set({ error: message, isLoading: false });
    }
  },

  deleteSkill: async (groupId: string, skillId: string) => {
    try {
      await apiDeleteSkill(groupId, skillId);
      set((state) => {
        const updatedGroups = state.groups
          .map((group) => {
            if (group.id !== groupId) return group;
            const remainingSkills = group.skills.filter(
              (s) => s.id !== skillId,
            );
            if (remainingSkills.length < 2) return null;
            return { ...group, skills: remainingSkills };
          })
          .filter(Boolean) as DedupGroup[];
        return { groups: updatedGroups };
      });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to delete skill from group';
      set({ error: message });
      throw err;
    }
  },
}));

export { useDedupStore };
