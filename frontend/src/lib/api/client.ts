import axios, { AxiosError } from "axios";
import Cookies from "js-cookie";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080/api/v1";

export const api = axios.create({
  baseURL: API_URL,
  headers: {
    "Content-Type": "application/json",
  },
});

let activeOrgId: string | null = null;

export function setActiveOrgId(orgId: string | null) {
  activeOrgId = orgId;
  if (orgId) {
    Cookies.set("active_org_id", orgId, { expires: 30 });
  } else {
    Cookies.remove("active_org_id");
  }
}

export function getActiveOrgId(): string | null {
  if (activeOrgId) return activeOrgId;
  return Cookies.get("active_org_id") || null;
}

api.interceptors.request.use(
  (config) => {
    const token = Cookies.get("access_token");
    if (token) {
      config.headers = config.headers || {};
      config.headers["Authorization"] = `Bearer ${token}`;
    }
    const orgId = getActiveOrgId();
    if (orgId) {
      config.headers = config.headers || {};
      config.headers["X-Organization-Id"] = orgId;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

api.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config;
    if (error.response?.status === 401 && originalRequest && !originalRequest._retry) {
      originalRequest._retry = true;
      try {
        const refreshToken = Cookies.get("refresh_token");
        if (!refreshToken) throw new Error("No refresh token found");

        const response = await axios.post(`${API_URL}/auth/refresh`, {
          refresh_token: refreshToken,
        });

        const payload = response.data?.data ?? response.data;
        const accessToken = payload.access_token ?? payload.accessToken;
        const newRefreshToken = payload.refresh_token ?? payload.refreshToken;

        if (!accessToken) throw new Error("No access token in refresh response");

        Cookies.set("access_token", accessToken, { expires: 1 / 96 });
        if (newRefreshToken) {
          Cookies.set("refresh_token", newRefreshToken, { expires: 30 });
        }

        originalRequest.headers = originalRequest.headers || {};
        originalRequest.headers["Authorization"] = `Bearer ${accessToken}`;
        return api(originalRequest);
      } catch (refreshError) {
        Cookies.remove("access_token");
        Cookies.remove("refresh_token");
        if (typeof window !== "undefined") {
          window.location.href = "/login";
        }
        return Promise.reject(refreshError);
      }
    }
    return Promise.reject(error);
  }
);

export function getErrorMessage(error: unknown, fallback = "Something went wrong"): string {
  if (axios.isAxiosError(error)) {
    const ax = error as AxiosError<{ error?: { message?: string } | string; message?: string }>;
    const data = ax.response?.data;
    if (data?.error) {
      if (typeof data.error === "string") return data.error;
      if (data.error.message) return data.error.message;
    }
    if (data?.message) return data.message;
    return ax.message || fallback;
  }
  if (error instanceof Error) return error.message;
  return fallback;
}
