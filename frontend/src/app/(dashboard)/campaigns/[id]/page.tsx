"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { 
  ArrowLeft, 
  AlertTriangle,
  RefreshCw,
  Tag,
  AlertCircle
} from "lucide-react";
import { 
  ResponsiveContainer, 
  PieChart, 
  Pie, 
  Cell, 
  Legend, 
  Tooltip 
} from "recharts";
import { Campaign, Message } from "@/types";

interface CampaignData {
  id: string;
  name: string;
  type: string;
  status: string;
  total_recipient_count: number;
  sent_count: number;
  delivered_count: number;
  read_count: number;
  failed_count: number;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
}

interface MessageData {
  id: string;
  contact_id: string;
  wa_message_id: string | null;
  status: string;
  failure_reason: string | null;
  sent_at: string | null;
  delivered_at: string | null;
  read_at: string | null;
}

export default function CampaignDetails() {
  const params = useParams();
  const campaignId = params.id as string;

  const [campaign, setCampaign] = useState<CampaignData | null>(null);
  const [messages, setMessages] = useState<MessageData[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const fetchCampaignDetails = async () => {
    try {
      const { api } = await import("@/lib/api");
      const [campRes, msgRes] = await Promise.allSettled([
        api.get(`/campaigns/${campaignId}`),
        api.get(`/campaigns/${campaignId}/messages`)
      ]);

      if (campRes.status === "fulfilled") {
        setCampaign(campRes.value.data.data);
      } else {
        throw new Error(campRes.reason?.response?.data?.error?.message || "Failed to load campaign");
      }
      
      if (msgRes.status === "fulfilled") {
        setMessages(msgRes.value.data.data.data || []);
      }
      
      setError("");
    } catch (e: any) {
      const msg = e.message || "Failed to load campaign details";
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (campaignId) fetchCampaignDetails();
  }, [campaignId]);

  // Auto-refresh polling if running
  useEffect(() => {
    let interval: NodeJS.Timeout;
    if (campaign?.status === "running") {
      interval = setInterval(fetchCampaignDetails, 3000);
    }
    return () => {
      if (interval) clearInterval(interval);
    };
  }, [campaign?.status]);

  if (loading) {
    return (
      <div className="flex justify-center items-center py-20">
        <div className="w-8 h-8 border-4 border-primary/20 border-t-primary rounded-full animate-spin" />
      </div>
    );
  }

  if (error || !campaign) {
    return (
      <div className="glass-panel p-10 rounded-2xl text-center text-muted-foreground border border-white/5 space-y-4">
        <AlertCircle className="w-8 h-8 text-rose-400 mx-auto" />
        <p>{error || "Campaign not found"}</p>
        <Link href="/campaigns" className="text-primary hover:underline block">
          Go back to campaigns list.
        </Link>
      </div>
    );
  }

  const pieData = [
    { name: "Delivered (Unread)", value: (campaign.delivered_count || 0) - (campaign.read_count || 0), color: "#10b981" },
    { name: "Read", value: campaign.read_count || 0, color: "#38bdf8" },
    { name: "Sent (Undelivered)", value: (campaign.sent_count || 0) - (campaign.delivered_count || 0), color: "#6b7280" },
    { name: "Failed", value: campaign.failed_count || 0, color: "#f43f5e" }
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
                campaign.status === "failed" ? "bg-rose-500/10 text-rose-400 border border-rose-500/20" :
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
          <p className="text-2xl font-bold text-white">{(campaign.total_recipient_count || 0).toLocaleString()}</p>
        </div>
        <div className="glass-panel p-5 rounded-2xl">
          <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">Delivered</p>
          <p className="text-2xl font-bold text-emerald-400">{(campaign.delivered_count || 0).toLocaleString()}</p>
        </div>
        <div className="glass-panel p-5 rounded-2xl">
          <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">Read Rate</p>
          <p className="text-2xl font-bold text-sky-400">
            {(campaign.sent_count || 0) > 0 ? `${Math.round(((campaign.read_count || 0) / (campaign.sent_count || 1)) * 100)}%` : "0%"}
          </p>
        </div>
        <div className="glass-panel p-5 rounded-2xl">
          <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">Failed</p>
          <p className="text-2xl font-bold text-rose-400">{(campaign.failed_count || 0).toLocaleString()}</p>
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
                <span className="font-bold text-white">{new Date(campaign.created_at).toLocaleString()}</span>
              </div>
              <div className="p-3.5 bg-white/2 border border-white/5 rounded-xl">
                <span className="text-muted-foreground uppercase font-semibold block mb-1">Started At</span>
                <span className="font-bold text-white">{campaign.started_at ? new Date(campaign.started_at).toLocaleString() : "Pending"}</span>
              </div>
              <div className="p-3.5 bg-white/2 border border-white/5 rounded-xl">
                <span className="text-muted-foreground uppercase font-semibold block mb-1">Finished At</span>
                <span className="font-bold text-white">{campaign.completed_at ? new Date(campaign.completed_at).toLocaleString() : "Running"}</span>
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
              <span className="font-bold text-white block">Audience Target</span>
              <span className="text-muted-foreground mt-0.5 block">This campaign broadcasted to target segments.</span>
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
                      Receiver (ID: {msg.contact_id.slice(0, 5)})
                    </td>
                    <td className="px-6 py-4 text-xs text-muted-foreground font-mono">{msg.wa_message_id || "Pending"}</td>
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
                      {msg.read_at ? `Read: ${new Date(msg.read_at).toLocaleTimeString()}` :
                       msg.delivered_at ? `Deliv: ${new Date(msg.delivered_at).toLocaleTimeString()}` :
                       msg.sent_at ? `Sent: ${new Date(msg.sent_at).toLocaleTimeString()}` : "Pending"}
                    </td>
                    <td className="px-6 py-4 text-xs text-right">
                      {msg.status === "failed" ? (
                        <div className="flex items-center justify-end gap-1.5 text-rose-400">
                          <AlertTriangle className="w-3.5 h-3.5" />
                          <span className="truncate max-w-xs">{msg.failure_reason}</span>
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
