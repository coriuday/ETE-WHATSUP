import { api } from "./client";
import type { User } from "@/types";

export interface AuthTokens {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

function mapUser(raw: Record<string, unknown>): User {
  return {
    id: String(raw.id),
    email: String(raw.email),
    fullName:
      (raw.full_name as string) ||
      (raw.fullName as string) ||
      `${raw.first_name ?? ""} ${raw.last_name ?? ""}`.trim(),
    role: (raw.role as User["role"]) || "team_member",
    isEmailVerified: Boolean(raw.email_verified ?? raw.isEmailVerified),
    twoFactorEnabled: Boolean(raw.two_factor_enabled ?? raw.twoFactorEnabled),
    createdAt: String(raw.created_at ?? raw.createdAt ?? ""),
    updatedAt: String(raw.updated_at ?? raw.updatedAt ?? ""),
  };
}

export async function fetchMe(): Promise<User> {
  const res = await api.get("/auth/me");
  const data = res.data?.data?.user ?? res.data?.data ?? res.data;
  return mapUser(data);
}

export async function loginRequest(email: string, password: string, totpCode?: string) {
  const res = await api.post("/auth/login", {
    email,
    password,
    totp_code: totpCode,
  });
  return res.data?.data ?? res.data;
}
