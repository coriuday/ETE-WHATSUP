"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { PageHeader, DataTable, StatusBadge, ErrorState, EmptyState } from "@/components/ui";

export default function SchedulesPage() {
  const q = useQuery({
    queryKey: ["schedules"],
    queryFn: async () => (await api.get("/schedules")).data?.data?.schedules || [],
  });
  if (q.isError) return <ErrorState onRetry={() => q.refetch()} />;
  const items = q.data || [];
  return (
    <div className="space-y-6">
      <PageHeader title="Schedules" description="Campaign schedules and follow-ups." />
      {!items.length ? <EmptyState title="No schedules" description="Schedule a campaign from the campaign wizard." /> : (
        <DataTable headers={["Campaign", "Frequency", "Next run", "Status"]}>
          {items.map((s: { id: string; campaign_name: string; frequency: string; next_run_at: string; status: string }) => (
            <tr key={s.id} className="border-b border-border">
              <td className="px-3 py-2">{s.campaign_name}</td>
              <td className="px-3 py-2">{s.frequency}</td>
              <td className="px-3 py-2">{s.next_run_at}</td>
              <td className="px-3 py-2"><StatusBadge status={s.status} /></td>
            </tr>
          ))}
        </DataTable>
      )}
    </div>
  );
}
