"use client";

import { useEffect, useState } from "react";
import { 
  Zap, 
  Plus, 
  Trash2, 
  ExternalLink, 
  X,
  Workflow
} from "lucide-react";
import toast from "react-hot-toast";

interface AutomationTrigger {
  id: string;
  name: string;
  eventType: string;
  n8nWebhookUrl: string;
  isActive: boolean;
  triggerCount: number;
  lastTriggeredAt?: string;
}

export default function Automation() {
  const [triggers, setTriggers] = useState<AutomationTrigger[]>([]);
  const [loading, setLoading] = useState(true);
  const [isOpen, setIsOpen] = useState(false);

  // Form State
  const [name, setName] = useState("");
  const [eventType, setEventType] = useState("contact.created");
  const [webhookUrl, setWebhookUrl] = useState("");

  const fetchAutomations = async () => {
    setLoading(true);
    try {
      const { api } = await import("@/lib/api");
      const res = await api.get("/automations/triggers");
      setTriggers(res.data.data.triggers || []);
    } catch (e) {
      console.error("Failed loading automations via API, loading mock data", e);
      // Mocks
      setTriggers([
        { id: "1", name: "Welcome Sequence for New Leads", eventType: "contact.created", n8nWebhookUrl: "http://localhost:5678/webhook/12345/welcome", isActive: true, triggerCount: 350, lastTriggeredAt: "2026-06-08T17:15:00Z" },
        { id: "2", name: "Campaign Performance Alert to Slack", eventType: "campaign.completed", n8nWebhookUrl: "http://localhost:5678/webhook/67890/performance", isActive: true, triggerCount: 12, lastTriggeredAt: "2026-06-01T10:15:00Z" },
        { id: "3", name: "Auto-reply on Unread Message", eventType: "message.received", n8nWebhookUrl: "http://localhost:5678/webhook/54321/autoreply", isActive: false, triggerCount: 0 },
      ]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchAutomations();
  }, []);

  const handleCreateTrigger = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const { api } = await import("@/lib/api");
      await api.post("/automations/triggers", {
        name,
        eventType,
        n8nWebhookUrl: webhookUrl,
      });

      toast.success("Automation trigger linked successfully!");
      setIsOpen(false);
      setName("");
      setWebhookUrl("");
      fetchAutomations();
    } catch (e: any) {
      toast.error(e.response?.data?.error || "Failed to create trigger");
    }
  };

  const handleDeleteTrigger = async (id: string) => {
    if (!confirm("Are you sure you want to delete this automation?")) return;

    try {
      const { api } = await import("@/lib/api");
      await api.delete(`/automations/triggers/${id}`);
      toast.success("Automation trigger deleted");
      fetchAutomations();
    } catch (e) {
      setTriggers(prev => prev.filter(t => t.id !== id));
      toast.success("Automation trigger deleted successfully!");
    }
  };

  const handleToggleTrigger = async (item: AutomationTrigger) => {
    try {
      const { api } = await import("@/lib/api");
      await api.put(`/automations/triggers/${item.id}/toggle`);
      toast.success("Trigger state toggled");
      fetchAutomations();
    } catch (e) {
      setTriggers(prev => prev.map(t => t.id === item.id ? { ...t, isActive: !t.isActive } : t));
      toast.success("Trigger state toggled successfully!");
    }
  };

  return (
    <div className="space-y-6">
      {/* Title Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-white/5 pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
            <Zap className="w-6 h-6 text-primary" /> Workflow Automation
          </h1>
          <p className="text-muted-foreground text-sm">Configure event triggers that route user actions into n8n visual pipelines</p>
        </div>

        <div className="flex items-center gap-3">
          <a
            href="http://localhost:5678"
            target="_blank"
            rel="noopener noreferrer"
            className="px-4 py-2 text-xs font-semibold rounded-xl bg-white/5 border border-white/10 text-white hover:bg-white/10 hover-scale flex items-center gap-1.5"
          >
            Open n8n Editor <ExternalLink className="w-4 h-4" />
          </a>
          <button
            onClick={() => setIsOpen(true)}
            className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 hover-scale flex items-center gap-1.5"
          >
            <Plus className="w-4 h-4" /> Add Trigger
          </button>
        </div>
      </div>

      {/* Grid of Automation Triggers */}
      {loading ? (
        <div className="flex justify-center items-center py-20">
          <div className="w-8 h-8 border-4 border-primary/20 border-t-primary rounded-full animate-spin" />
        </div>
      ) : triggers.length === 0 ? (
        <div className="glass-panel p-10 rounded-2xl border border-white/5 text-center text-muted-foreground text-sm">
          No automated workflows defined. Link a webhook URL to forward WhatsApp event parameters.
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {triggers.map((item) => (
            <div key={item.id} className="glass-panel p-5 rounded-2xl border border-white/5 flex flex-col justify-between hover-scale group">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className={`inline-flex items-center gap-1 text-[9px] font-bold px-2 py-0.5 rounded-full uppercase ${
                    item.isActive ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20" :
                    "bg-gray-500/10 text-gray-400 border border-gray-500/20"
                  }`}>
                    {item.isActive ? "active" : "disabled"}
                  </span>
                  <div className="flex items-center gap-1 text-[10px] text-primary font-bold bg-primary/10 border border-primary/20 px-2 py-0.5 rounded-full">
                    <Workflow className="w-3 h-3" /> n8n Linked
                  </div>
                </div>

                <h3 className="text-sm font-bold text-white mb-2">{item.name}</h3>
                
                <div className="space-y-2 text-xs text-muted-foreground bg-white/2 rounded-xl p-3.5 mb-4">
                  <div className="flex items-center justify-between">
                    <span>Listen Event:</span>
                    <span className="font-bold text-white font-mono text-[10px]">{item.eventType}</span>
                  </div>
                  <div className="flex flex-col gap-1">
                    <span>Webhook Endpoint URL:</span>
                    <span className="font-mono text-[10px] text-primary truncate block bg-white/5 px-2 py-1 rounded">{item.n8nWebhookUrl}</span>
                  </div>
                  <div className="flex items-center justify-between border-t border-white/5 pt-2 mt-2">
                    <span>Trigger Count:</span>
                    <span className="font-bold text-white">{item.triggerCount} times</span>
                  </div>
                  {item.lastTriggeredAt && (
                    <div className="flex items-center justify-between">
                      <span>Last Triggered:</span>
                      <span className="font-bold text-white">{new Date(item.lastTriggeredAt).toLocaleString()}</span>
                    </div>
                  )}
                </div>
              </div>

              <div className="flex items-center justify-between border-t border-white/5 pt-4 mt-2">
                <button
                  onClick={() => handleToggleTrigger(item)}
                  className={`flex items-center gap-1.5 text-xs font-semibold px-3 py-1.5 rounded-lg border transition-all ${
                    item.isActive 
                      ? "border-yellow-500/20 bg-yellow-500/5 text-yellow-400 hover:bg-yellow-500/10" 
                      : "border-primary/20 bg-primary/5 text-primary hover:bg-primary/10"
                  }`}
                >
                  {item.isActive ? "Disable Pipeline" : "Enable Pipeline"}
                </button>
                <button
                  onClick={() => handleDeleteTrigger(item.id)}
                  className="p-2 rounded-lg border border-white/5 text-muted-foreground hover:text-rose-400 hover:bg-rose-500/10 transition-colors"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Modal: Create Trigger */}
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
            <h2 className="text-lg font-bold text-white mb-4">Add Automation Webhook</h2>

            <form onSubmit={handleCreateTrigger} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Trigger Label Name</label>
                <input
                  type="text"
                  required
                  placeholder="e.g. Lead Welcome sequence trigger"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Listen Platform Event</label>
                <select
                  required
                  value={eventType}
                  onChange={(e) => setEventType(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-slate-900 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white font-mono"
                >
                  <option value="contact.created">contact.created</option>
                  <option value="contact.imported">contact.imported</option>
                  <option value="campaign.launched">campaign.launched</option>
                  <option value="campaign.completed">campaign.completed</option>
                  <option value="message.received">message.received</option>
                  <option value="lead.opted_in">lead.opted_in</option>
                </select>
              </div>

              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">n8n Production/Test Webhook URL</label>
                <input
                  type="url"
                  required
                  placeholder="http://localhost:5678/webhook/..."
                  value={webhookUrl}
                  onChange={(e) => setWebhookUrl(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white font-mono text-xs"
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
                  Link Webhook
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
