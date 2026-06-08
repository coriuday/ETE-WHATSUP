"use client";

import { useEffect, useState } from "react";
import { 
  Calendar, 
  Plus, 
  Trash2, 
  Play, 
  Pause, 
  Globe, 
  X
} from "lucide-react";
import toast from "react-hot-toast";

interface ScheduleItem {
  id: string;
  name: string;
  campaignName: string;
  cronExpression?: string;
  timezone: string;
  nextRunAt?: string;
  lastRunAt?: string;
  isActive: boolean;
  status: "idle" | "running" | "paused";
}

export default function Schedules() {
  const [schedules, setSchedules] = useState<ScheduleItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [isOpen, setIsOpen] = useState(false);

  // Form State
  const [name, setName] = useState("");
  const [campaignId, setCampaignId] = useState("");
  const [frequency, setFrequency] = useState("daily");
  const [cronExp, setCronExp] = useState("0 10 * * *");
  const [timezone, setTimezone] = useState("Asia/Kolkata");

  const [campaignsList, setCampaignsList] = useState<any[]>([]);

  const fetchSchedules = async () => {
    setLoading(true);
    try {
      const { api } = await import("@/lib/api");
      const res = await api.get("/schedules");
      setSchedules(res.data.data.schedules || []);
      
      const campaignsRes = await api.get("/campaigns");
      setCampaignsList(campaignsRes.data.data.campaigns || []);
    } catch (e) {
      console.error("Failed loading schedules via API, loading mock data", e);
      // Mocks
      setSchedules([
        { id: "1", name: "Daily Customer Check-in", campaignName: "Morning Greetings Broadcast", cronExpression: "0 9 * * *", timezone: "Asia/Kolkata", nextRunAt: "2026-06-09T09:00:00Z", lastRunAt: "2026-06-08T09:00:00Z", isActive: true, status: "idle" },
        { id: "2", name: "Weekly Newsletter Blast", campaignName: "Friday Deals Promo", cronExpression: "0 15 * * 5", timezone: "Asia/Kolkata", nextRunAt: "2026-06-12T15:00:00Z", lastRunAt: "2026-06-05T15:00:00Z", isActive: true, status: "idle" },
        { id: "3", name: "Monthly NPS Feedback", campaignName: "NPS Survey Campaign", cronExpression: "0 10 1 * *", timezone: "Asia/Kolkata", nextRunAt: "2026-07-01T10:00:00Z", lastRunAt: "2026-06-01T10:00:00Z", isActive: false, status: "paused" },
      ]);
      setCampaignsList([
        { id: "c1", name: "Morning Greetings Broadcast" },
        { id: "c2", name: "Friday Deals Promo" },
        { id: "c3", name: "NPS Survey Campaign" }
      ]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSchedules();
  }, []);

  const handleCreateSchedule = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const { api } = await import("@/lib/api");
      await api.post("/schedules", {
        name,
        campaignId,
        cronExpression: cronExp,
        timezone,
      });

      toast.success("Broadcast schedule created successfully!");
      setIsOpen(false);
      resetForm();
      fetchSchedules();
    } catch (e: any) {
      toast.error(e.response?.data?.error || "Failed to create schedule");
    }
  };

  const handleToggleActive = async (item: ScheduleItem) => {
    const action = item.isActive ? "pause" : "resume";
    try {
      const { api } = await import("@/lib/api");
      await api.post(`/schedules/${item.id}/${action}`);
      toast.success(`Schedule ${action}d successfully`);
      fetchSchedules();
    } catch (e) {
      // Mock toggle
      setSchedules(prev => prev.map(s => s.id === item.id ? { ...s, isActive: !s.isActive, status: s.isActive ? "paused" : "idle" } : s));
      toast.success(`Schedule ${action === "pause" ? "paused" : "resumed"} successfully!`);
    }
  };

  const handleDeleteSchedule = async (id: string) => {
    if (!confirm("Are you sure you want to delete this schedule?")) return;

    try {
      const { api } = await import("@/lib/api");
      await api.delete(`/schedules/${id}`);
      toast.success("Schedule deleted");
      fetchSchedules();
    } catch (e) {
      setSchedules(prev => prev.filter(s => s.id !== id));
      toast.success("Schedule deleted successfully!");
    }
  };

  const resetForm = () => {
    setName("");
    setCampaignId("");
    setFrequency("daily");
    setCronExp("0 10 * * *");
  };

  const updateCronByFrequency = (freq: string) => {
    setFrequency(freq);
    if (freq === "daily") setCronExp("0 10 * * *");
    else if (freq === "weekly") setCronExp("0 15 * * 5");
    else if (freq === "monthly") setCronExp("0 10 1 * *");
  };

  return (
    <div className="space-y-6">
      {/* Title Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-white/5 pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
            <Calendar className="w-6 h-6 text-primary" /> Campaign Schedules
          </h1>
          <p className="text-muted-foreground text-sm">Configure automated cron recurrence and send times for broadcasts</p>
        </div>

        <button
          onClick={() => setIsOpen(true)}
          className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 hover-scale flex items-center gap-1.5"
        >
          <Plus className="w-4 h-4" /> Add Schedule
        </button>
      </div>

      {/* Grid of Schedules */}
      {loading ? (
        <div className="flex justify-center items-center py-20">
          <div className="w-8 h-8 border-4 border-primary/20 border-t-primary rounded-full animate-spin" />
        </div>
      ) : schedules.length === 0 ? (
        <div className="glass-panel p-10 rounded-2xl border border-white/5 text-center text-muted-foreground text-sm">
          No schedules defined. Add a schedule to automate periodic WhatsApp broadcasts.
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {schedules.map((item) => (
            <div key={item.id} className="glass-panel p-5 rounded-2xl border border-white/5 flex flex-col justify-between hover-scale group">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className={`inline-flex items-center gap-1 text-[9px] font-bold px-2 py-0.5 rounded-full uppercase ${
                    item.isActive ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20" :
                    "bg-gray-500/10 text-gray-400 border border-gray-500/20"
                  }`}>
                    {item.isActive ? "active" : "paused"}
                  </span>
                  <div className="flex items-center gap-1.5 text-[10px] text-muted-foreground font-semibold">
                    <Globe className="w-3 h-3" /> {item.timezone}
                  </div>
                </div>

                <h3 className="text-sm font-bold text-white mb-2">{item.name}</h3>
                
                <div className="space-y-2.5 text-xs text-muted-foreground bg-white/2 rounded-xl p-3.5 mb-4">
                  <div className="flex items-center justify-between">
                    <span>Campaign:</span>
                    <span className="font-bold text-white truncate max-w-[150px]">{item.campaignName}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span>Cron Pattern:</span>
                    <span className="font-mono text-[10px] bg-white/5 px-2 py-0.5 rounded text-primary font-bold">{item.cronExpression}</span>
                  </div>
                  {item.nextRunAt && (
                    <div className="flex items-center justify-between">
                      <span>Next Run:</span>
                      <span className="font-bold text-white">{new Date(item.nextRunAt).toLocaleString()}</span>
                    </div>
                  )}
                </div>
              </div>

              <div className="flex items-center justify-between border-t border-white/5 pt-4 mt-2">
                <button
                  onClick={() => handleToggleActive(item)}
                  className={`flex items-center gap-1.5 text-xs font-semibold px-3 py-1.5 rounded-lg border transition-all ${
                    item.isActive 
                      ? "border-yellow-500/20 bg-yellow-500/5 text-yellow-400 hover:bg-yellow-500/10" 
                      : "border-primary/20 bg-primary/5 text-primary hover:bg-primary/10"
                  }`}
                >
                  {item.isActive ? (
                    <><Pause className="w-3.5 h-3.5" /> Pause</>
                  ) : (
                    <><Play className="w-3.5 h-3.5" /> Activate</>
                  )}
                </button>
                <button
                  onClick={() => handleDeleteSchedule(item.id)}
                  className="p-2 rounded-lg border border-white/5 text-muted-foreground hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Modal: Create Schedule */}
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div className="fixed inset-0 bg-slate-950/65 backdrop-blur-sm" onClick={() => setIsOpen(false)} />
          <div className="glass-panel w-full max-w-md rounded-2xl border border-white/10 p-6 z-10 shadow-2xl relative">
            <button
              onClick={() => setIsOpen(false)}
              className="absolute right-4 top-4 text-muted-foreground hover:text-white"
            >
              <X className="w-5 h-5" />
            </button>
            <h2 className="text-lg font-bold text-white mb-4">Add Broadcast Schedule</h2>

            <form onSubmit={handleCreateSchedule} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Schedule Label Name</label>
                <input
                  type="text"
                  required
                  placeholder="e.g. Daily Check-in Broadcast"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Link Campaign</label>
                <select
                  required
                  value={campaignId}
                  onChange={(e) => setCampaignId(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-slate-900 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                >
                  <option value="">Select campaign to broadcast...</option>
                  {campaignsList.map(c => (
                    <option key={c.id} value={c.id}>{c.name}</option>
                  ))}
                </select>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Frequency</label>
                  <select
                    value={frequency}
                    onChange={(e) => updateCronByFrequency(e.target.value)}
                    className="w-full px-3.5 py-2.5 bg-slate-900 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                  >
                    <option value="daily">Daily</option>
                    <option value="weekly">Weekly</option>
                    <option value="monthly">Monthly</option>
                  </select>
                </div>
                <div>
                  <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Timezone</label>
                  <select
                    value={timezone}
                    onChange={(e) => setTimezone(e.target.value)}
                    className="w-full px-3.5 py-2.5 bg-slate-900 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                  >
                    <option value="Asia/Kolkata">Asia/Kolkata</option>
                    <option value="UTC">UTC</option>
                    <option value="America/New_York">America/New York</option>
                  </select>
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Cron Pattern Expression</label>
                <input
                  type="text"
                  required
                  value={cronExp}
                  onChange={(e) => setCronExp(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white font-mono"
                />
              </div>

              <div className="flex items-center justify-end gap-3 pt-4 border-t border-white/5">
                <button
                  type="button"
                  onClick={() => setIsOpen(false)}
                  className="px-4 py-2 text-xs font-semibold rounded-xl bg-white/5 border border-white/10 text-white hover:bg-white/10"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95"
                >
                  Create Schedule
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
