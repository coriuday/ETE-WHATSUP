"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { 
  ArrowLeft, 
  AlertTriangle,
  RefreshCw,
  Tag
} from "lucide-react";
import { 
  ResponsiveContainer, 
  PieChart, 
  Pie, 
  Cell, 
  Legend, 
  Tooltip 
} from "recharts";
import toast from "react-hot-toast";
import { Campaign, Message } from "@/types";

export default function CampaignDetails() {
  const params = useParams();
  const campaignId = params.id as string;

  const [campaign, setCampaign] = useState<Campaign | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchCampaignDetails = async () => {
    setLoading(true);
    try {
      const { api } = await import("@/lib/api");
      const campRes = await api.get(`/campaigns/${campaignId}`);
      setCampaign(campRes.data.data.campaign);
      
      const msgRes = await api.get(`/messages?campaignId=${campaignId}`);
      setMessages(msgRes.data.data.messages || []);
    } catch (e) {
      console.error("Failed loading campaign details via API, loading mock data", e);
      // Mocks
      setCampaign({
        id: campaignId,
        organizationId: "1",
        waAccountId: "acc1",
        name: "Summer Blast Broadcast",
        type: "promotional",
        status: "completed",
        totalRecipientCount: 1500,
        sentCount: 1500,
        deliveredCount: 1420,
        readCount: 1150,
        failedCount: 80,
        createdAt: "2026-06-02T10:00:00Z",
        updatedAt: "2026-06-02T10:00:00Z",
        startedAt: "2026-06-02T10:02:00Z",
        completedAt: "2026-06-02T10:15:00Z"
      });

      setMessages([
        { id: "m1", organizationId: "1", waAccountId: "acc1", campaignId, contactId: "c1", waMessageId: "wamid.1", direction: "outbound", type: "template", body: "Hello Rahul, check out our summer discounts!", status: "read", sentAt: "2026-06-02T10:02:05Z", deliveredAt: "2026-06-02T10:02:10Z", readAt: "2026-06-02T10:05:00Z", createdAt: "" },
        { id: "m2", organizationId: "1", waAccountId: "acc1", campaignId, contactId: "c2", waMessageId: "wamid.2", direction: "outbound", type: "template", body: "Hello Priya, check out our summer discounts!", status: "delivered", sentAt: "2026-06-02T10:02:12Z", deliveredAt: "2026-06-02T10:02:18Z", createdAt: "" },
        { id: "m3", organizationId: "1", waAccountId: "acc1", campaignId, contactId: "c3", waMessageId: "wamid.3", direction: "outbound", type: "template", body: "Hello Amit, check out our summer discounts!", status: "failed", sentAt: "2026-06-02T10:02:15Z", failedAt: "2026-06-02T10:02:20Z", failureReason: "Undeliverable number / User has blocked notifications", createdAt: "" },
        { id: "m4", organizationId: "1", waAccountId: "acc1", campaignId, contactId: "c4", waMessageId: "wamid.4", direction: "outbound", type: "template", body: "Hello Sarah, check out our summer discounts!", status: "sent", sentAt: "2026-06-02T10:02:22Z", createdAt: "" }
      ]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (campaignId) fetchCampaignDetails();
  }, [campaignId]);

  if (loading) {
    return (
      <div className="flex justify-center items-center py-20">
        <div className="w-8 h-8 border-4 border-primary/20 border-t-primary rounded-full animate-spin" />
      </div>
    );
  }

  if (!campaign) {
    return (
      <div className="glass-panel p-10 rounded-2xl text-center text-muted-foreground border border-white/5">
        Campaign not found.{" "}
        <Link href="/campaigns" className="text-primary hover:underline">
          Go back to campaigns list.
        </Link>
      </div>
    );
  }

  const pieData = [
    { name: "Delivered (Unread)", value: campaign.deliveredCount - campaign.readCount, color: "#10b981" },
    { name: "Read", value: campaign.readCount, color: "#38bdf8" },
    { name: "Sent (Undelivered)", value: campaign.sentCount - campaign.deliveredCount, color: "#6b7280" },
    { name: "Failed", value: campaign.failedCount, color: "#f43f5e" }
  ].filter(d => d.value > 0);

  return (
    <div className="space-y-6">
      {/* Title Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-white/5 pb-5">
        <div className="flex items-center gap-3">
          <Link
            href="/campaigns"
            className="p-2 rounded-xl bg-white/5 border border-white/10 text-muted-foreground hover:text-white"
          >
            <ArrowLeft className="w-4.5 h-4.5" />
          </Link>
          <div>
            <h1 className="text-2xl font-bold tracking-tight text-white">{campaign.name}</h1>
            <p className="text-muted-foreground text-sm flex items-center gap-1.5 mt-0.5">
              <span>Status:</span>
              <span className={`inline-block text-[9px] font-bold px-2 py-0.5 rounded-full uppercase ${
                campaign.status === "completed" ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20" :
                campaign.status === "running" ? "bg-primary/10 text-primary border border-primary/20 animate-pulse" :
                "bg-yellow-500/10 text-yellow-400 border border-yellow-500/20"
              }`}>
                {campaign.status}
              </span>
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={fetchCampaignDetails}
            className="p-2.5 rounded-xl bg-white/5 border border-white/10 text-muted-foreground hover:text-white hover:bg-white/10"
          >
            <RefreshCw className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* KPI Cards Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-5">
        <div className="glass-panel p-5 rounded-2xl">
          <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">Total Targeted</p>
          <p className="text-2xl font-bold text-white">{campaign.totalRecipientCount.toLocaleString()}</p>
        </div>
        <div className="glass-panel p-5 rounded-2xl">
          <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">Delivered</p>
          <p className="text-2xl font-bold text-emerald-400">{campaign.deliveredCount.toLocaleString()}</p>
        </div>
        <div className="glass-panel p-5 rounded-2xl">
          <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">Read Rate</p>
          <p className="text-2xl font-bold text-sky-400">
            {campaign.sentCount > 0 ? `${Math.round((campaign.readCount / campaign.sentCount) * 100)}%` : "0%"}
          </p>
        </div>
        <div className="glass-panel p-5 rounded-2xl">
          <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">Failed</p>
          <p className="text-2xl font-bold text-rose-400">{campaign.failedCount.toLocaleString()}</p>
        </div>
      </div>

      {/* Charts & Timeline */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Pie Chart */}
        <div className="glass-panel p-6 rounded-2xl border border-white/5 flex flex-col items-center">
          <h3 className="text-sm font-bold text-white mb-6 align-self-start">Delivery Funnel Distribution</h3>
          
          <div className="h-[220px] w-full relative">
            {pieData.length > 0 ? (
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie
                    data={pieData}
                    cx="50%"
                    cy="50%"
                    innerRadius={60}
                    outerRadius={80}
                    paddingAngle={3}
                    dataKey="value"
                  >
                    {pieData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={entry.color} />
                    ))}
                  </Pie>
                  <Tooltip 
                    contentStyle={{ background: "#0d1423", border: "1px solid rgba(255,255,255,0.08)", borderRadius: "12px" }}
                    itemStyle={{ fontSize: 11 }}
                  />
                  <Legend 
                    verticalAlign="bottom" 
                    iconSize={10} 
                    iconType="circle"
                    wrapperStyle={{ fontSize: 10, color: "#fff" }}
                  />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <div className="absolute inset-0 flex items-center justify-center text-muted-foreground text-xs">
                No delivery stats available.
              </div>
            )}
          </div>
        </div>

        {/* Campaign Info */}
        <div className="lg:col-span-2 glass-panel p-6 rounded-2xl border border-white/5 flex flex-col justify-between">
          <div>
            <h3 className="text-sm font-bold text-white mb-4">Metadata & Timeline</h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs">
              <div className="p-3.5 bg-white/2 border border-white/5 rounded-xl">
                <span className="text-muted-foreground uppercase font-semibold block mb-1">Created At</span>
                <span className="font-bold text-white">{new Date(campaign.createdAt).toLocaleString()}</span>
              </div>
              <div className="p-3.5 bg-white/2 border border-white/5 rounded-xl">
                <span className="text-muted-foreground uppercase font-semibold block mb-1">Started At</span>
                <span className="font-bold text-white">{campaign.startedAt ? new Date(campaign.startedAt).toLocaleString() : "Pending"}</span>
              </div>
              <div className="p-3.5 bg-white/2 border border-white/5 rounded-xl">
                <span className="text-muted-foreground uppercase font-semibold block mb-1">Finished At</span>
                <span className="font-bold text-white">{campaign.completedAt ? new Date(campaign.completedAt).toLocaleString() : "Running"}</span>
              </div>
              <div className="p-3.5 bg-white/2 border border-white/5 rounded-xl">
                <span className="text-muted-foreground uppercase font-semibold block mb-1">WABA Sender Linked</span>
                <span className="font-bold text-primary">Connected Meta Phone</span>
              </div>
            </div>
          </div>

          <div className="mt-6 p-4 bg-primary/5 border border-primary/10 rounded-xl flex items-center gap-3">
            <Tag className="w-5 h-5 text-primary" />
            <div className="text-xs">
              <span className="font-bold text-white block">Audience Tag Targets</span>
              <span className="text-muted-foreground mt-0.5 block">This campaign broadcasted to members matching segment tags.</span>
            </div>
          </div>
        </div>
      </div>

      {/* Messages Logs Table */}
      <div className="glass-panel rounded-2xl border border-white/5 overflow-hidden">
        <div className="px-6 py-4 border-b border-white/5">
          <h3 className="text-sm font-bold text-white">Broadcast Delivery Logs</h3>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="border-b border-white/5 bg-white/2">
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Receiver Contact</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Meta Message ID</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Status</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider">Timestamps</th>
                <th className="px-6 py-4 text-xs font-semibold text-muted-foreground uppercase tracking-wider text-right">Details</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/5">
              {messages.length === 0 ? (
                <tr>
                  <td colSpan={5} className="px-6 py-10 text-center text-muted-foreground text-xs">
                    No message delivery receipts returned yet.
                  </td>
                </tr>
              ) : (
                messages.map((msg) => (
                  <tr key={msg.id} className="hover:bg-white/2 transition-colors">
                    <td className="px-6 py-4 text-sm font-semibold text-white">
                      {/* Placeholder name mapping */}
                      Receiver (ID: {msg.contactId.slice(0, 5)})
                    </td>
                    <td className="px-6 py-4 text-xs text-muted-foreground font-mono">{msg.waMessageId || "Pending"}</td>
                    <td className="px-6 py-4">
                      <span className={`inline-block text-[9px] font-bold px-2 py-0.5 rounded-full uppercase ${
                        msg.status === "read" ? "bg-sky-500/10 text-sky-400 border border-sky-500/20" :
                        msg.status === "delivered" ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20" :
                        msg.status === "failed" ? "bg-rose-500/10 text-rose-400 border border-rose-500/20" :
                        "bg-gray-500/10 text-gray-400 border border-gray-500/20"
                      }`}>
                        {msg.status}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-xs text-muted-foreground">
                      {msg.readAt ? `Read: ${new Date(msg.readAt).toLocaleTimeString()}` :
                       msg.deliveredAt ? `Deliv: ${new Date(msg.deliveredAt).toLocaleTimeString()}` :
                       msg.sentAt ? `Sent: ${new Date(msg.sentAt).toLocaleTimeString()}` : "Pending"}
                    </td>
                    <td className="px-6 py-4 text-xs text-right">
                      {msg.status === "failed" ? (
                        <div className="flex items-center justify-end gap-1.5 text-rose-400">
                          <AlertTriangle className="w-3.5 h-3.5" />
                          <span className="truncate max-w-xs">{msg.failureReason}</span>
                        </div>
                      ) : "Delivered successfully"}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
