import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface User {
  userid: string;
  name: string;
  email: string;
  balance: string; // decimal string
  quantity: string;
}

interface AuthState {
  token: string | null;
  user: User | null;
  setAuth: (token: string, user: User) => void;
  logout: () => void;
  updateUser: (user: User) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      token: null,
      user: null,
      setAuth: (token, user) => set({ token, user }),
      logout: () => set({ token: null, user: null }),
      updateUser: (user) => set({ user }),
    }),
    {
      name: 'perp-auth-storage',
    }
  )
);
