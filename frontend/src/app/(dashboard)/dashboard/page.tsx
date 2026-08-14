"use client";

import { useQuery } from "@tanstack/react-query";
import { Area, AreaChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { PageHeader, StatCard, ChartCard, ErrorState, LoadingSkeleton, StatusBadge } from "@/components/ui";
import { getOverview } from "@/lib/api/analytics";
import { listCampaigns } from "@/lib/api/campaigns";
import { listConversations } from "@/lib/api/inbox";
import { api } from "@/lib/api";
import Link from "next/link";

export default function DashboardPage() {
  const overview = useQuery({ queryKey: ["analytics-overview"], queryFn: getOverview });
  const campaigns = useQuery({
    queryKey: ["campaigns-recent"],
    queryFn: () => listCampaigns({ limit: 5 }),
  });
  const convos = useQuery({
    queryKey: ["inbox-recent"],
    queryFn: () => listConversations({}),
  });
  const wa = useQuery({
    queryKey: ["wa-accounts"],
    queryFn: async () => (await api.get("/whatsapp/accounts")).data?.data,
  });
  const autos = useQuery({
    queryKey: ["automation-runs"],
    queryFn: async () => (await api.get("/automations/runs")).data?.data,
  });

  if (overview.isError) {
    return <ErrorState message="Could not load dashboard" onRetry={() => overview.refetch()} />;
  }

  const data = overview.data || {};
  const messages = data.messages || {};
  const contacts = data.contacts || {};
  const series = data.timeseries || [];
  const campaignList = campaigns.data?.items || campaigns.data?.campaigns || [];
  const conversations = convos.data?.conversations || [];
  const accounts = wa.data?.accounts || [];
  const runs = autos.data?.runs || [];

  return (
    <div className="space-y-6">
      <PageHeader title="Dashboard" description="Operational visibility for this workspace." />
      {overview.isLoading ? (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <LoadingSkeleton key={i} className="h-24" />
          ))}
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <StatCard label="Contacts" value={contacts.total ?? 0} />
          <StatCard label="Messages sent" value={messages.total_sent ?? 0} />
          <StatCard label="Delivery rate" value={`${Number(messages.delivery_rate || 0).toFixed(1)}%`} />
          <StatCard label="Read rate" value={`${Number(messages.read_rate || 0).toFixed(1)}%`} hint={`${data.campaigns?.running ?? 0} campaigns running`} />
        </div>
      )}

      <div className="grid gap-4 lg:grid-cols-3">
        <div className="lg:col-span-2">
          <ChartCard title="Campaign activity">
            <div className="h-56">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={series}>
                  <XAxis dataKey="day" hide />
                  <YAxis hide />
                  <Tooltip />
                  <Area type="monotone" dataKey="sent" stroke="hsl(var(--primary))" fill="hsl(var(--primary) / 0.15)" />
                </AreaChart>
              </ResponsiveContainer>
            </div>
          </ChartCard>
        </div>
        <div className="rounded-xl border border-border bg-card p-4">
          <h3 className="mb-3 text-sm font-semibold">WhatsApp connection</h3>
          {accounts.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              Provider not configured. Connect WhatsApp to enable live messaging. Mock provider is available for Alpha.
            </p>
          ) : (
            <ul className="space-y-2 text-sm">
              {accounts.slice(0, 4).map((a: { id: string; display_name: string; provider?: string; status?: string }) => (
                <li key={a.id} className="flex items-center justify-between">
                  <span>{a.display_name}</span>
                  <StatusBadge status={a.provider || a.status || "mock"} />
                </li>
              ))}
            </ul>
          )}
          <Link href="/whatsapp" className="mt-3 inline-block text-sm text-primary">
            Manage accounts
          </Link>
        </div>
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <div className="rounded-xl border border-border bg-card p-4">
          <h3 className="mb-3 text-sm font-semibold">Recent conversations</h3>
          <ul className="space-y-2 text-sm">
            {conversations.slice(0, 6).map((c: { id: string; last_message_body?: string; contact?: { name?: string } }) => (
              <li key={c.id}>
                <Link href="/inbox" className="hover:text-primary">
                  <span className="font-medium">{c.contact?.name || "Unknown"}</span>
                  <span className="ml-2 text-muted-foreground">{c.last_message_body}</span>
                </Link>
              </li>
            ))}
            {!conversations.length && <p className="text-muted-foreground">No conversations yet.</p>}
          </ul>
        </div>
        <div className="rounded-xl border border-border bg-card p-4">
          <h3 className="mb-3 text-sm font-semibold">Campaigns</h3>
          <ul className="space-y-2 text-sm">
            {(Array.isArray(campaignList) ? campaignList : []).slice(0, 6).map((c: { id: string; name: string; status: string }) => (
              <li key={c.id} className="flex items-center justify-between">
                <Link href={`/campaigns/${c.id}`}>{c.name}</Link>
                <StatusBadge status={c.status} />
              </li>
            ))}
          </ul>
          <h3 className="mb-3 mt-6 text-sm font-semibold">Automation activity</h3>
          <ul className="space-y-2 text-sm text-muted-foreground">
            {runs.slice(0, 5).map((r: { id: string; trigger_type: string; status: string }) => (
              <li key={r.id} className="flex justify-between">
                <span>{r.trigger_type}</span>
                <StatusBadge status={r.status} />
              </li>
            ))}
            {!runs.length && <li>No automation runs yet.</li>}
          </ul>
        </div>
      </div>
    </div>
  );
}
