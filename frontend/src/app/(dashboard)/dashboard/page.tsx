"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { 
  Send, 
  CheckCircle2, 
  Eye, 
  AlertTriangle, 
  PlusCircle, 
  ArrowUpRight, 
  TrendingUp,
  MessageSquare
} from "lucide-react";
import { 
  ResponsiveContainer, 
  AreaChart, 
  Area, 
  XAxis, 
  YAxis, 
  Tooltip, 
} from "recharts";
import { useAuthStore } from "@/store/authStore";

interface OverviewStats {
  sent_count: number;
  delivered_count: number;
  read_count: number;
  failed_count: number;
}

interface Campaign {
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
}

export default function Dashboard() {
  const { user } = useAuthStore();
  const [stats, setStats] = useState<OverviewStats>({
    sent_count: 0,
    delivered_count: 0,
    read_count: 0,
    failed_count: 0,
  });
  
  const [recentCampaigns, setRecentCampaigns] = useState<Campaign[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    const fetchOverview = async () => {
      try {
        const { api } = await import("@/lib/api");
        
        // Use Promise.allSettled so one failure doesn't block the other
        const [statsRes, campaignsRes] = await Promise.allSettled([
          api.get("/analytics/overview"),
          api.get("/campaigns?limit=5")
        ]);

        if (statsRes.status === "fulfilled") {
          setStats({
            sent_count: statsRes.value.data.data?.sent_count || 0,
            delivered_count: statsRes.value.data.data?.delivered_count || 0,
            read_count: statsRes.value.data.data?.read_count || 0,
            failed_count: statsRes.value.data.data?.failed_count || 0,
          });
        }

        if (campaignsRes.status === "fulfilled") {
          setRecentCampaigns(campaignsRes.value.data.data?.data?.slice(0, 5) || []);
        }
      } catch (e) {
        console.error("Error fetching dashboard statistics", e);
      } finally {
        setIsLoading(false);
      }
    };
    fetchOverview();
  }, []);

  const total = stats.sent_count || 0;
  const readRate = total > 0 ? ((stats.read_count / total) * 100).toFixed(1) : "0.0";
  const deliveryRate = total > 0 ? ((stats.delivered_count / total) * 100).toFixed(1) : "0.0";
  const errorRate = total > 0 ? ((stats.failed_count / total) * 100).toFixed(1) : "0.0";

  // Mock chart data since backend doesn't have timeseries endpoint yet
  const chartData = [
    { name: "Mon", Sent: 0, Delivered: 0, Read: 0 },
    { name: "Tue", Sent: 0, Delivered: 0, Read: 0 },
    { name: "Wed", Sent: 0, Delivered: 0, Read: 0 },
    { name: "Thu", Sent: Math.max(0, stats.sent_count - 50), Delivered: Math.max(0, stats.delivered_count - 50), Read: Math.max(0, stats.read_count - 50) },
    { name: "Fri", Sent: stats.sent_count, Delivered: stats.delivered_count, Read: stats.read_count },
    { name: "Sat", Sent: 0, Delivered: 0, Read: 0 },
    { name: "Sun", Sent: 0, Delivered: 0, Read: 0 },
  ];

  return (
    <div className="space-y-6">
      {/* Title Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-white/5 pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white">Dashboard Overview</h1>
          <p className="text-muted-foreground text-sm">Real-time performance metrics and communications analytics</p>
        </div>

        <div className="flex items-center gap-3">
          <Link
            href="/contacts"
            className="px-4 py-2 text-xs font-semibold rounded-xl bg-white/5 border border-white/10 text-white hover:bg-white/10 hover-scale"
          >
            Import Contacts
          </Link>
          <Link
            href="/campaigns"
            className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 hover-scale flex items-center gap-1.5"
          >
            <PlusCircle className="w-4 h-4" /> Create Campaign
          </Link>
        </div>
      </div>

      {/* KPI Cards Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-5">
        {/* Sent */}
        <div className="glass-panel p-5 rounded-2xl relative overflow-hidden group">
          <div className="absolute top-0 right-0 w-24 h-24 bg-primary/5 rounded-full blur-2xl pointer-events-none group-hover:bg-primary/10 transition-colors"></div>
          <div className="flex items-center justify-between mb-4">
            <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Messages Sent</span>
            <div className="w-8 h-8 rounded-lg bg-primary/10 border border-primary/20 flex items-center justify-center text-primary">
              <Send className="w-4 h-4" />
            </div>
          </div>
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-bold text-white">{stats.sent_count.toLocaleString()}</span>
            <span className="text-xs text-primary font-bold flex items-center gap-0.5">
              <TrendingUp className="w-3.5 h-3.5" /> All time
            </span>
          </div>
        </div>

        {/* Delivered */}
        <div className="glass-panel p-5 rounded-2xl relative overflow-hidden group">
          <div className="absolute top-0 right-0 w-24 h-24 bg-emerald-500/5 rounded-full blur-2xl pointer-events-none group-hover:bg-emerald-500/10 transition-colors"></div>
          <div className="flex items-center justify-between mb-4">
            <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Delivered</span>
            <div className="w-8 h-8 rounded-lg bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
              <CheckCircle2 className="w-4 h-4" />
            </div>
          </div>
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-bold text-white">{stats.delivered_count.toLocaleString()}</span>
            <span className="text-xs text-emerald-400 font-bold">{deliveryRate}% Rate</span>
          </div>
        </div>

        {/* Read */}
        <div className="glass-panel p-5 rounded-2xl relative overflow-hidden group">
          <div className="absolute top-0 right-0 w-24 h-24 bg-sky-500/5 rounded-full blur-2xl pointer-events-none group-hover:bg-sky-500/10 transition-colors"></div>
          <div className="flex items-center justify-between mb-4">
            <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Read Rate</span>
            <div className="w-8 h-8 rounded-lg bg-sky-500/10 border border-sky-500/20 flex items-center justify-center text-sky-400">
              <Eye className="w-4 h-4" />
            </div>
          </div>
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-bold text-white">{readRate}%</span>
            <span className="text-xs text-sky-400 font-bold">{stats.read_count.toLocaleString()} Messages</span>
          </div>
        </div>

        {/* Failed */}
        <div className="glass-panel p-5 rounded-2xl relative overflow-hidden group">
          <div className="absolute top-0 right-0 w-24 h-24 bg-rose-500/5 rounded-full blur-2xl pointer-events-none group-hover:bg-rose-500/10 transition-colors"></div>
          <div className="flex items-center justify-between mb-4">
            <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Failed Delivery</span>
            <div className="w-8 h-8 rounded-lg bg-rose-500/10 border border-rose-500/20 flex items-center justify-center text-rose-400">
              <AlertTriangle className="w-4 h-4" />
            </div>
          </div>
          <div className="flex items-baseline gap-2">
            <span className="text-2xl font-bold text-white">{stats.failed_count.toLocaleString()}</span>
            <span className="text-xs text-rose-400 font-bold">Error rate: {errorRate}%</span>
          </div>
        </div>
      </div>

      {/* Main Charts & Side Bar */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Main Area Chart */}
        <div className="lg:col-span-2 glass-panel p-6 rounded-2xl border border-white/5">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h3 className="text-sm font-bold text-white">Daily Outflow Delivery</h3>
              <p className="text-xs text-muted-foreground mt-0.5">Campaign performance timeline</p>
            </div>
            <div className="flex items-center gap-3">
              <span className="inline-flex items-center gap-1 text-[11px] font-semibold text-primary">
                <span className="w-2.5 h-2.5 rounded-full bg-primary/40 border border-primary"></span> Sent
              </span>
              <span className="inline-flex items-center gap-1 text-[11px] font-semibold text-sky-400">
                <span className="w-2.5 h-2.5 rounded-full bg-sky-400/40 border border-sky-400"></span> Read
              </span>
            </div>
          </div>

          <div className="h-[300px] w-full">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
                <defs>
                  <linearGradient id="colorSent" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="hsl(var(--primary))" stopOpacity={0.25} />
                    <stop offset="95%" stopColor="hsl(var(--primary))" stopOpacity={0} />
                  </linearGradient>
                  <linearGradient id="colorRead" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#38bdf8" stopOpacity={0.25} />
                    <stop offset="95%" stopColor="#38bdf8" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <XAxis dataKey="name" tickLine={false} axisLine={false} tick={{ fill: "rgba(255,255,255,0.4)", fontSize: 10 }} />
                <YAxis tickLine={false} axisLine={false} tick={{ fill: "rgba(255,255,255,0.4)", fontSize: 10 }} />
                <Tooltip 
                  contentStyle={{ background: "#0d1423", border: "1px solid rgba(255,255,255,0.08)", borderRadius: "12px" }}
                  labelStyle={{ color: "#fff", fontSize: 12, fontWeight: "bold" }}
                  itemStyle={{ fontSize: 11 }}
                />
                <Area type="monotone" dataKey="Sent" stroke="hsl(var(--primary))" strokeWidth={2} fillOpacity={1} fill="url(#colorSent)" />
                <Area type="monotone" dataKey="Read" stroke="#38bdf8" strokeWidth={2} fillOpacity={1} fill="url(#colorRead)" />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Side Panel: Recent Campaigns */}
        <div className="glass-panel p-6 rounded-2xl flex flex-col justify-between border border-white/5">
          <div>
            <div className="flex items-center justify-between mb-5">
              <h3 className="text-sm font-bold text-white">Recent Campaigns</h3>
              <Link href="/campaigns" className="text-xs font-semibold text-primary hover:underline flex items-center gap-0.5">
                View All <ArrowUpRight className="w-3.5 h-3.5" />
              </Link>
            </div>

            <div className="space-y-4">
              {recentCampaigns.length === 0 ? (
                <div className="text-center text-xs text-muted-foreground py-4">
                  No campaigns run yet.
                </div>
              ) : (
                recentCampaigns.map((camp) => (
                  <div key={camp.id} className="p-3.5 rounded-xl bg-white/5 border border-white/5 flex items-start justify-between gap-3 hover:bg-white/10 transition-colors">
                    <div className="min-w-0">
                      <p className="text-xs font-bold text-white truncate">{camp.name}</p>
                      <p className="text-[10px] text-muted-foreground uppercase tracking-wider mt-1">{camp.type}</p>
                    </div>
                    <div className="text-right flex-shrink-0">
                      <span className={`inline-block text-[9px] font-bold px-2 py-0.5 rounded-full uppercase ${
                        camp.status === "completed" ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20" :
                        camp.status === "running" ? "bg-primary/10 text-primary border border-primary/20 animate-pulse" :
                        camp.status === "failed" ? "bg-rose-500/10 text-rose-400 border border-rose-500/20" :
                        "bg-yellow-500/10 text-yellow-400 border border-yellow-500/20"
                      }`}>
                        {camp.status}
                      </span>
                      <p className="text-[10px] text-muted-foreground mt-1 font-semibold">
                        {camp.sent_count || 0} / {camp.total_recipient_count || 0}
                      </p>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>

          <div className="mt-6 p-4 rounded-xl bg-primary/5 border border-primary/10 flex items-center gap-3.5">
            <MessageSquare className="w-5 h-5 text-primary flex-shrink-0" />
            <div className="min-w-0">
              <p className="text-[11px] font-bold text-white">Support & Verification</p>
              <p className="text-[10px] text-muted-foreground mt-0.5">Link WhatsApp Accounts to start broadcasting.</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
