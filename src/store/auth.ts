import { create } from "zustand";
import type { UserInfo } from "../types";

interface AuthState {
  user: UserInfo | null;
  setUser: (u: UserInfo | null) => void;
  logout: () => void;
}

export const useAuth = create<AuthState>((set) => ({
  user: null,
  setUser: (u) => set({ user: u }),
  logout: () => set({ user: null }),
}));
