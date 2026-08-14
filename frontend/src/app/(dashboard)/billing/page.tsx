"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { useAuthStore } from "@/store/authStore";
import { PageHeader, StatCard, ComingSoon, ErrorState } from "@/components/ui";
import { useSearchParams } from "next/navigation";

export default function BillingPage() {
  const tab = useSearchParams().get("tab");
  const orgId = useAuthStore((s) => s.activeOrgId || s.organization?.id);
  const q = useQuery({
    queryKey: ["usage", orgId],
    enabled: !!orgId,
    queryFn: async () => (await api.get(`/organizations/${orgId}/usage`)).data?.data,
  });
  if (q.isError) return <ErrorState onRetry={() => q.refetch()} />;
  if (tab === "invoices") return <ComingSoon title="Invoices" description="Stripe invoicing is out of scope for Alpha." />;
  return (
    <div className="space-y-6">
      <PageHeader title="Plan & usage" description="Live usage from the workspace. Checkout is coming soon." />
      <div className="grid gap-4 sm:grid-cols-2">
        <StatCard label="Contacts" value={`${q.data?.contacts?.used ?? 0} / ${q.data?.contacts?.limit ?? "—"}`} />
        <StatCard label="Messages this month" value={`${q.data?.messages?.used_this_month ?? 0} / ${q.data?.messages?.limit ?? "—"}`} />
      </div>
    </div>
  );
}
