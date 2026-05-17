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
  currentPage: number;
  totalPages: number;
  total: number;
  perPage: number;
  search: (query: string, category?: string, page?: number, perPage?: number) => Promise<void>;
  fetchByCategory: (category: string) => Promise<void>;
  incrementDownloads: (packageId: string) => void;
}

const useMarketStore = create<MarketState>((set, get) => ({
  skills: [],
  categories: [],
  isLoading: false,
  error: null,
  currentPage: 1,
  totalPages: 1,
  total: 0,
  perPage: 15,

  get isEmpty() {
    return get().skills.length === 0 && !get().isLoading;
  },

  search: async (query: string, category?: string, page?: number, perPage?: number) => {
    const p = page ?? 1;
    const pp = perPage ?? get().perPage;
    set({ isLoading: true, error: null, currentPage: p });
    try {
      const result = await searchMarket(query, category, p, pp);
      set({
        skills: result.skills,
        total: result.total,
        totalPages: Math.max(1, Math.ceil(result.total / pp)),
        isLoading: false,
      });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to search market';
      set({ error: message, isLoading: false });
    }
  },

  fetchByCategory: async (category: string) => {
    set({ isLoading: true, error: null, currentPage: 1 });
    try {
      const result = await searchMarket('', category, 1, get().perPage);
      set({
        skills: result.skills,
        total: result.total,
        totalPages: Math.max(1, Math.ceil(result.total / get().perPage)),
        isLoading: false,
      });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to fetch category';
      set({ error: message, isLoading: false });
    }
  },

  incrementDownloads: (packageId: string) => {
    set((state) => ({
      skills: state.skills.map((s) =>
        s.packageId === packageId ? { ...s, downloads: s.downloads + 1 } : s,
      ),
    }));
  },
}));

export { useMarketStore };
