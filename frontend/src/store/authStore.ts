import { create } from "zustand";
import Cookies from "js-cookie";
import { User, Organization } from "@/types";

interface AuthState {
  user: User | null;
  organization: Organization | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  setUser: (user: User | null) => void;
  setOrganization: (org: Organization | null) => void;
  setTokens: (accessToken: string, refreshToken: string) => void;
  logout: () => void;
  initialize: () => Promise<void>;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  user: null,
  organization: null,
  isAuthenticated: false,
  isLoading: true,

  setUser: (user) => set({ user, isAuthenticated: !!user }),
  setOrganization: (organization) => set({ organization }),

  setTokens: (accessToken, refreshToken) => {
    Cookies.set("access_token", accessToken, { expires: 1 / 96 }); // 15 mins
    Cookies.set("refresh_token", refreshToken, { expires: 30 }); // 30 days
  },

  logout: () => {
    Cookies.remove("access_token");
    Cookies.remove("refresh_token");
    set({ user: null, organization: null, isAuthenticated: false });
    if (typeof window !== "undefined") {
      window.location.href = "/login";
    }
  },

  initialize: async () => {
    set({ isLoading: true });
    const accessToken = Cookies.get("access_token");
    if (!accessToken) {
      set({ isLoading: false, isAuthenticated: false, user: null });
      return;
    }

    try {
      // Fetch user profile from API
      // Since api is configured with interceptor, it will attach the token
      const { api } = await import("@/lib/api");
      const userRes = await api.get("/auth/me");
      const user: User = userRes.data.data.user;
      
      let organization: Organization | null = null;
      try {
        const orgRes = await api.get("/organizations");
        const orgs: Organization[] = orgRes.data.data.organizations || [];
        if (orgs.length > 0) {
          organization = orgs[0]; // Set first organization as default
        }
      } catch (e) {
        console.error("Failed to load organizations", e);
      }

      set({
        user,
        organization,
        isAuthenticated: true,
        isLoading: false,
      });
    } catch (error) {
      console.error("Failed to initialize auth state", error);
      Cookies.remove("access_token");
      Cookies.remove("refresh_token");
      set({ user: null, organization: null, isAuthenticated: false, isLoading: false });
    }
  },
}));
