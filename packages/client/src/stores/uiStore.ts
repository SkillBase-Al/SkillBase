import { create } from 'zustand';
import type { AppSettings } from '../types/settings';

interface UIState {
  sidebarCollapsed: boolean;
  theme: AppSettings['theme'];
  toggleSidebar: () => void;
  setTheme: (theme: AppSettings['theme']) => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
}

const useUIStore = create<UIState>((set) => ({
  sidebarCollapsed: false,
  theme: 'system',

  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

  setTheme: (theme) => set({ theme }),

  setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
}));

export { useUIStore };
