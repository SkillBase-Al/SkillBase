import { create } from 'zustand';
import type { AgentConfig } from '../types/agent';
import {
  getAgents,
  addAgent as apiAddAgent,
  updateAgent as apiUpdateAgent,
  deleteAgent as apiDeleteAgent,
} from '../services/tauri';

interface AgentState {
  agents: AgentConfig[];
  isLoading: boolean;
  error: string | null;
  isEmpty: boolean;
  fetchAgents: () => Promise<void>;
  addAgent: (name: string, agentType: string, basePath: string) => Promise<void>;
  updateAgent: (id: string, name: string, agentType: string, basePath: string) => Promise<void>;
  deleteAgent: (agentId: string) => Promise<void>;
}

const useAgentStore = create<AgentState>((set, get) => ({
  agents: [],
  isLoading: false,
  error: null,
  get isEmpty() {
    return get().agents.length === 0 && !get().isLoading;
  },

  fetchAgents: async () => {
    set({ isLoading: true, error: null });
    try {
      const agents = await getAgents();
      set({ agents, isLoading: false });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to fetch agents';
      set({ error: message, isLoading: false });
    }
  },

  addAgent: async (name, agentType, basePath) => {
    set({ isLoading: true, error: null });
    try {
      const agent = await apiAddAgent(name, agentType, basePath);
      set((state) => ({
        agents: [...state.agents, agent],
        isLoading: false,
      }));
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to add agent';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  updateAgent: async (id, name, agentType, basePath) => {
    set({ isLoading: true, error: null });
    try {
      await apiUpdateAgent(id, name, agentType, basePath);
      set((state) => ({
        agents: state.agents.map((a) =>
          a.id === id ? { ...a, name, agentType, basePath } : a,
        ),
        isLoading: false,
      }));
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to update agent';
      set({ error: message, isLoading: false });
      throw err;
    }
  },

  deleteAgent: async (agentId) => {
    set({ isLoading: true, error: null });
    try {
      await apiDeleteAgent(agentId);
      set((state) => ({
        agents: state.agents.filter((a) => a.id !== agentId),
        isLoading: false,
      }));
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to delete agent';
      set({ error: message, isLoading: false });
      throw err;
    }
  },
}));

export { useAgentStore };
