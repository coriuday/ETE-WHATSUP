"use client";

import { useQuery } from "@tanstack/react-query";
import { useSearchParams } from "next/navigation";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/authStore";
import { PageHeader, DataTable, ErrorState, EmptyState } from "@/components/ui";

export default function TeamPage() {
  const tab = useSearchParams().get("tab") || "members";
  const orgId = useAuthStore((s) => s.activeOrgId || s.organization?.id);
  const members = useQuery({
    queryKey: ["members", orgId],
    enabled: !!orgId,
    queryFn: async () => (await api.get(`/organizations/${orgId}/members`)).data?.data?.members || [],
  });
  const audit = useQuery({
    queryKey: ["audit", orgId],
    enabled: !!orgId && tab === "audit",
    queryFn: async () => (await api.get(`/organizations/${orgId}/audit`)).data?.data?.events || [],
  });

  if (members.isError) return <ErrorState onRetry={() => members.refetch()} />;

  return (
    <div className="space-y-6">
      <PageHeader title="Team" description="Members, roles, and audit history." />
      {tab === "audit" ? (
        !audit.data?.length ? <EmptyState title="No audit events" /> : (
          <DataTable headers={["Action", "Resource", "When"]}>
            {audit.data.map((e: { id: string; action: string; resource_type?: string; created_at: string }) => (
              <tr key={e.id} className="border-b border-border">
                <td className="px-3 py-2">{e.action}</td>
                <td className="px-3 py-2">{e.resource_type}</td>
                <td className="px-3 py-2">{e.created_at}</td>
              </tr>
            ))}
          </DataTable>
        )
      ) : !members.data?.length ? (
        <EmptyState title="No members" />
      ) : (
        <DataTable headers={["Email", "Role"]}>
          {members.data.map((m: { user_id?: string; id?: string; email?: string; role?: string; user?: { email: string } }) => (
            <tr key={m.id || m.user_id} className="border-b border-border">
              <td className="px-3 py-2">{m.email || m.user?.email}</td>
              <td className="px-3 py-2">{m.role}</td>
            </tr>
          ))}
        </DataTable>
      )}
    </div>
  );
}
