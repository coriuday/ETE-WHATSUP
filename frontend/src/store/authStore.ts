import { create } from "zustand";
import Cookies from "js-cookie";
import { User, Organization } from "@/types";
import { setActiveOrgId, getActiveOrgId } from "@/lib/api";
import { fetchMe, listOrganizations } from "@/lib/api";

interface AuthState {
  user: User | null;
  organization: Organization | null;
  activeOrgId: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  hasOrganization: boolean;
  setUser: (user: User | null) => void;
  setOrganization: (org: Organization | null) => void;
  setTokens: (accessToken: string, refreshToken: string) => void;
  logout: () => void;
  initialize: () => Promise<void>;
}

export const useAuthStore = create<AuthState>((set) => ({
  user: null,
  organization: null,
  activeOrgId: getActiveOrgId(),
  isAuthenticated: false,
  isLoading: true,
  hasOrganization: false,

  setUser: (user) => set({ user, isAuthenticated: !!user }),

  setOrganization: (organization) => {
    const id = organization?.id ?? null;
    setActiveOrgId(id);
    set({
      organization,
      activeOrgId: id,
      hasOrganization: !!organization,
    });
  },

  setTokens: (accessToken, refreshToken) => {
    Cookies.set("access_token", accessToken, { expires: 1 / 96 });
    Cookies.set("refresh_token", refreshToken, { expires: 30 });
  },

  logout: () => {
    Cookies.remove("access_token");
    Cookies.remove("refresh_token");
    setActiveOrgId(null);
    set({
      user: null,
      organization: null,
      activeOrgId: null,
      isAuthenticated: false,
      hasOrganization: false,
    });
    if (typeof window !== "undefined") {
      window.location.href = "/login";
    }
  },

  initialize: async () => {
    set({ isLoading: true });
    const accessToken = Cookies.get("access_token");
    if (!accessToken) {
      set({
        isLoading: false,
        isAuthenticated: false,
        user: null,
        hasOrganization: false,
      });
      return;
    }

    try {
      const user = await fetchMe();
      let organization: Organization | null = null;
      try {
        const orgs = await listOrganizations();
        const savedId = getActiveOrgId();
        organization =
          orgs.find((o) => o.id === savedId) || orgs[0] || null;
        if (organization) {
          setActiveOrgId(organization.id);
        }
      } catch (e) {
        console.error("Failed to load organizations", e);
      }

      set({
        user,
        organization,
        activeOrgId: organization?.id ?? null,
        isAuthenticated: true,
        hasOrganization: !!organization,
        isLoading: false,
      });
    } catch (error) {
      console.error("Failed to initialize auth state", error);
      Cookies.remove("access_token");
      Cookies.remove("refresh_token");
      setActiveOrgId(null);
      set({
        user: null,
        organization: null,
        activeOrgId: null,
        isAuthenticated: false,
        hasOrganization: false,
        isLoading: false,
      });
    }
  },
}));
