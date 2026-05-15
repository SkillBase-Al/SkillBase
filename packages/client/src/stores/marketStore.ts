import { create } from 'zustand';
import type { MarketSkill } from '../types/skill';
import {
  searchMarket,
  getCategories,
} from '../services/tauri';

interface MarketState {
  skills: MarketSkill[];
  categories: string[];
  isLoading: boolean;
  error: string | null;
  isEmpty: boolean;
  search: (query: string, category?: string) => Promise<void>;
  fetchByCategory: (category: string) => Promise<void>;
}

const useMarketStore = create<MarketState>((set, get) => ({
  skills: [],
  categories: [],
  isLoading: false,
  error: null,
  get isEmpty() {
    return get().skills.length === 0 && !get().isLoading;
  },

  search: async (query: string, category?: string) => {
    set({ isLoading: true, error: null });
    try {
      const skills = await searchMarket(query, category);
      set({ skills, isLoading: false });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to search market';
      set({ error: message, isLoading: false });
    }
  },

  fetchByCategory: async (category: string) => {
    set({ isLoading: true, error: null });
    try {
      const skills = await searchMarket('', category);
      set({ skills, isLoading: false });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to fetch category';
      set({ error: message, isLoading: false });
    }
  },
}));

export { useMarketStore };
