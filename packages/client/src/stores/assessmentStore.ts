import { create } from 'zustand';
import type {
  FormatResult,
  SecurityResult,
  AssessmentSummary,
} from '../types/assessment';
import {
  assessFormat as apiAssessFormat,
  assessSecurity as apiAssessSecurity,
  batchAssess as apiBatchAssess,
  getAssessmentResults,
} from '../services/tauri';

interface AssessmentState {
  results: (FormatResult | SecurityResult)[];
  summary: AssessmentSummary | null;
  isLoading: boolean;
  error: string | null;
  assessFormat: (skillId: string) => Promise<FormatResult>;
  assessSecurity: (skillId: string) => Promise<SecurityResult>;
  batchAssess: () => Promise<void>;
  fetchResults: (skillId?: string) => Promise<void>;
  clearResults: () => void;
}

const useAssessmentStore = create<AssessmentState>((set) => ({
  results: [],
  summary: null,
  isLoading: false,
  error: null,

  assessFormat: async (skillId: string) => {
    set({ isLoading: true, error: null });
    try {
      const result = await apiAssessFormat(skillId);
      set((state) => ({
        results: [...state.results, result],
        isLoading: false,
      }));
      return result;
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to assess format';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  assessSecurity: async (skillId: string) => {
    set({ isLoading: true, error: null });
    try {
      const result = await apiAssessSecurity(skillId);
      set((state) => ({
        results: [...state.results, result],
        isLoading: false,
      }));
      return result;
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to assess security';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  batchAssess: async () => {
    set({ isLoading: true, error: null });
    try {
      const summary = await apiBatchAssess();
      set({ summary, isLoading: false });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to batch assess';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  fetchResults: async (skillId?: string) => {
    set({ isLoading: true, error: null });
    try {
      const results = await getAssessmentResults(skillId);
      set({ results, isLoading: false });
    } catch (err) {
      const message =
        err instanceof Error
          ? err.message
          : 'Failed to fetch assessment results';
      set({ error: message, isLoading: false });
    }
  },

  clearResults: () => set({ results: [], summary: null }),
}));

export { useAssessmentStore };
