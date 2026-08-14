"use client";

import { Suspense } from "react";
import { useQuery } from "@tanstack/react-query";
import { getOverview, getCampaignAnalytics } from "@/lib/api/analytics";
import { PageHeader, StatCard, ErrorState, DataTable, StatusBadge } from "@/components/ui";
import { useSearchParams } from "next/navigation";

export default function AnalyticsPage() {
  return (
    <Suspense fallback={null}>
      <AnalyticsPageInner />
    </Suspense>
  );
}

function AnalyticsPageInner() {
  const tab = useSearchParams().get("tab") || "overview";
  const overview = useQuery({ queryKey: ["analytics-overview"], queryFn: getOverview });
  const campaigns = useQuery({ queryKey: ["analytics-campaigns"], queryFn: getCampaignAnalytics });
  if (overview.isError) return <ErrorState onRetry={() => overview.refetch()} />;
  const m = overview.data?.messages || {};
  return (
    <div className="space-y-6">
      <PageHeader title="Analytics" description="Backend-driven metrics only." />
      <div className="grid gap-4 sm:grid-cols-4">
        <StatCard label="Sent" value={m.total_sent ?? 0} />
        <StatCard label="Delivered" value={m.total_delivered ?? 0} />
        <StatCard label="Read" value={m.total_read ?? 0} />
        <StatCard label="Failed" value={m.total_failed ?? 0} />
      </div>
      {(tab === "campaigns" || tab === "delivery") && (
        <DataTable headers={["Campaign", "Status", "Delivery rate"]}>
          {(campaigns.data?.campaigns || []).map((c: { id: string; name: string; status: string; delivery_rate: number }) => (
            <tr key={c.id} className="border-b border-border">
              <td className="px-3 py-2">{c.name}</td>
              <td className="px-3 py-2"><StatusBadge status={c.status} /></td>
              <td className="px-3 py-2">{Number(c.delivery_rate || 0).toFixed(1)}%</td>
            </tr>
          ))}
        </DataTable>
      )}
    </div>
  );
}
