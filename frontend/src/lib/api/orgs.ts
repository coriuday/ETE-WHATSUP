import { api } from "./client";
import type { Organization } from "@/types";

function mapOrg(raw: Record<string, unknown>): Organization {
  return {
    id: String(raw.id),
    name: String(raw.name),
    slug: String(raw.slug ?? ""),
    createdAt: String(raw.created_at ?? raw.createdAt ?? ""),
    updatedAt: String(raw.updated_at ?? raw.updatedAt ?? ""),
  };
}

export async function listOrganizations(): Promise<Organization[]> {
  const res = await api.get("/organizations");
  const payload = res.data?.data ?? res.data;
  const orgs = payload?.organizations ?? payload ?? [];
  return (Array.isArray(orgs) ? orgs : []).map(mapOrg);
}
