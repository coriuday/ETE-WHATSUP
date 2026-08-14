"use client";

import { useQuery } from "@tanstack/react-query";
import { useParams } from "next/navigation";
import { api } from "@/lib/api";
import { PageHeader, Tag, StatusBadge, ErrorState, LoadingSkeleton } from "@/components/ui";

export default function ContactDetailPage() {
  const { id } = useParams<{ id: string }>();
  const q = useQuery({
    queryKey: ["contact", id],
    queryFn: async () => (await api.get(`/contacts/${id}`)).data?.data,
  });
  if (q.isError) return <ErrorState onRetry={() => q.refetch()} />;
  if (q.isLoading) return <LoadingSkeleton className="h-40" />;
  const c = q.data || {};
  return (
    <div className="space-y-6">
      <PageHeader title={`${c.first_name || ""} ${c.last_name || ""}`.trim() || c.phone_number} description={c.phone_number} />
      <div className="grid gap-4 lg:grid-cols-3">
        <div className="rounded-xl border border-border bg-card p-4">
          <p className="text-sm">Email: {c.email || "—"}</p>
          <p className="mt-2 text-sm">Consent: {c.wa_opted_in ? "opted in" : "unknown"}</p>
          <StatusBadge status={c.wa_status || "active"} />
          <div className="mt-3 flex flex-wrap gap-1">{(c.tags || []).map((t: string) => <Tag key={t}>{t}</Tag>)}</div>
        </div>
        <div className="rounded-xl border border-border bg-card p-4 lg:col-span-2">
          <h3 className="text-sm font-semibold">Custom fields</h3>
          <pre className="mt-2 text-xs text-muted-foreground">{JSON.stringify(c.custom_fields || {}, null, 2)}</pre>
        </div>
      </div>
    </div>
  );
}
