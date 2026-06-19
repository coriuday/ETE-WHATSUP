"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import {
  Send,
  Plus,
  Clock,
  ChevronRight,
  Copy,
  X,
  AlertCircle,
  RefreshCw
} from "lucide-react";
import toast from "react-hot-toast";

interface CampaignItem {
  id: string;
  name: string;
  type: string;
  status: string;
  target_type: string;
  scheduled_at: string | null;
  created_at: string;
  updated_at: string;
  total_recipient_count?: number;
  sent_count?: number;
  delivered_count?: number;
  read_count?: number;
  failed_count?: number;
}

interface WaAccount {
  id: string;
  display_name: string;
}

interface Template {
  id: string;
  name: string;
  category: string;
  variables: string[];
}

export default function Campaigns() {
  const [campaigns, setCampaigns] = useState<CampaignItem[]>([]);
  const [waAccounts, setWaAccounts] = useState<WaAccount[]>([]);
  const [templates, setTemplates] = useState<Template[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Wizard State
  const [isWizardOpen, setIsWizardOpen] = useState(false);
  const [step, setStep] = useState(1);
  const [submitting, setSubmitting] = useState(false);

  // Form State
  const [name, setName] = useState("");
  const [type, setType] = useState<string>("promotional");
  const [selectedAccount, setSelectedAccount] = useState("");
  const [targetMode, setTargetMode] = useState<"segment" | "all_contacts">("all_contacts");
  const [selectedSegmentId, setSelectedSegmentId] = useState("");
  const [messageMode, setMessageMode] = useState<"template" | "text">("template");
  const [selectedTemplateId, setSelectedTemplateId] = useState("");
  const [messageBody, setMessageBody] = useState("");
  const [scheduleMode, setScheduleMode] = useState<"immediate" | "scheduled">("immediate");
  const [scheduledDate, setScheduledDate] = useState("");
  const [timezone, setTimezone] = useState(Intl.DateTimeFormat().resolvedOptions().timeZone);

  // Derived Template Variables
  const [templateVariables, setTemplateVariables] = useState<Record<string, string>>({});

  const fetchCampaignsAndConfig = async () => {
    setLoading(true);
    setError("");
    try {
      const { api } = await import("@/lib/api");
      
      const [campaignRes, accountRes, templateRes] = await Promise.allSettled([
        api.get("/campaigns"),
        api.get("/whatsapp/accounts"),
        api.get("/templates")
      ]);

      if (campaignRes.status === "fulfilled") {
        setCampaigns(campaignRes.value.data.data.data || []);
      } else {
        throw new Error(campaignRes.reason?.response?.data?.error?.message || "Failed to load campaigns");
      }
      
      if (accountRes.status === "fulfilled") {
        const accounts: WaAccount[] = accountRes.value.data.data.accounts || [];
        setWaAccounts(accounts);
        if (accounts.length > 0) setSelectedAccount(accounts[0].id);
      }

      if (templateRes.status === "fulfilled") {
        setTemplates(templateRes.value.data.data.data || []);
      }
    } catch (e: any) {
      const msg = e.message || "Failed to load campaigns data";
      setError(msg);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchCampaignsAndConfig();
  }, []);

  const handleTemplateChange = (id: string) => {
    setSelectedTemplateId(id);
    const selected = templates.find(t => t.id === id);
    if (selected && selected.variables) {
      const vars: Record<string, string> = {};
      selected.variables.forEach(v => {
        vars[v] = "";
      });
      setTemplateVariables(vars);
    } else {
      setTemplateVariables({});
    }
  };

  const handleLaunchCampaign = async () => {
    if (!selectedAccount) {
      toast.error("Please select a WhatsApp account");
      return;
    }

    setSubmitting(true);
    try {
      const { api } = await import("@/lib/api");
      
      // 1. Create Campaign
      const createRes = await api.post("/campaigns", {
        name,
        type,
        wa_account_id: selectedAccount,
        target_type: targetMode,
        target_segment_id: targetMode === "segment" ? selectedSegmentId : undefined,
        template_id: messageMode === "template" ? selectedTemplateId : undefined,
        message_body: messageMode === "text" ? messageBody : undefined,
        timezone,
      });

      const campaign = createRes.data.data;
      const campaignId = campaign.id;

      if (scheduleMode === "immediate") {
        // Launch Campaign Immediately
        await api.post(`/campaigns/${campaignId}/launch`);
        toast.success("Campaign launched successfully!");
      } else {
        await api.post(`/campaigns/${campaignId}/schedule`, {
          scheduled_at: new Date(scheduledDate).toISOString()
        });
        toast.success("Campaign scheduled successfully!");
      }

      setIsWizardOpen(false);
      resetWizard();
      fetchCampaignsAndConfig();
    } catch (e: any) {
      const msg = e.response?.data?.error?.message || "Failed to launch campaign";
      toast.error(typeof msg === "string" ? msg : "Failed to launch campaign");
    } finally {
      setSubmitting(false);
    }
  };

  const resetWizard = () => {
    setStep(1);
    setName("");
    setType("promotional");
    setTargetMode("all_contacts");
    setSelectedSegmentId("");
    setMessageMode("template");
    setSelectedTemplateId("");
    setMessageBody("");
    setScheduleMode("immediate");
    setScheduledDate("");
    setTemplateVariables({});
  };

  const handleCloneCampaign = async (id: string) => {
    try {
      const { api } = await import("@/lib/api");
      await api.post(`/campaigns/${id}/clone`);
      toast.success("Campaign cloned to draft!");
      fetchCampaignsAndConfig();
    } catch (e: any) {
      const msg = e.response?.data?.error?.message || "Failed to clone campaign";
      toast.error(typeof msg === "string" ? msg : "Failed to clone");
    }
  };

  return (
    <div className="space-y-6">
      {/* Title Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-white/5 pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
            <Send className="w-6 h-6 text-primary" /> Campaigns Manager
          </h1>
          <p className="text-muted-foreground text-sm">Send bulk message templates and track deliverability metrics</p>
        </div>

        <button
          onClick={() => {
            resetWizard();
            setIsWizardOpen(true);
          }}
          className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 hover-scale flex items-center gap-1.5"
        >
          <Plus className="w-4 h-4" /> Start Campaign
        </button>
      </div>

      {/* Campaigns Grid */}
      {loading ? (
        <div className="flex justify-center items-center py-20">
          <div className="w-8 h-8 border-4 border-primary/20 border-t-primary rounded-full animate-spin" />
        </div>
      ) : error ? (
        <div className="glass-panel p-10 rounded-2xl border border-white/5 text-center space-y-4">
          <AlertCircle className="w-8 h-8 text-rose-400 mx-auto" />
          <p className="text-muted-foreground text-sm">{error}</p>
          <button onClick={fetchCampaignsAndConfig} className="px-4 py-2 text-xs font-semibold rounded-xl bg-white/5 border border-white/10 text-white hover:bg-white/10">
            <RefreshCw className="w-3.5 h-3.5 inline mr-1.5" /> Retry
          </button>
        </div>
      ) : campaigns.length === 0 ? (
        <div className="glass-panel p-10 rounded-2xl border border-white/5 text-center text-muted-foreground text-sm">
          No campaigns triggered yet. Launch a new campaign to begin.
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {campaigns.map((camp) => (
            <div key={camp.id} className="glass-panel p-5 rounded-2xl border border-white/5 flex flex-col justify-between hover-scale group">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className={`inline-block text-[9px] font-bold px-2 py-0.5 rounded-full uppercase ${
                    camp.status === "completed" ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20" :
                    camp.status === "running" ? "bg-primary/10 text-primary border border-primary/20 animate-pulse" :
                    camp.status === "failed" ? "bg-rose-500/10 text-rose-400 border border-rose-500/20" :
                    "bg-yellow-500/10 text-yellow-400 border border-yellow-500/20"
                  }`}>
                    {camp.status}
                  </span>
                  <span className="text-[10px] text-muted-foreground font-semibold uppercase tracking-wider">{camp.type}</span>
                </div>

                <h3 className="text-sm font-bold text-white mb-2">{camp.name}</h3>
                
                {camp.status === "scheduled" && camp.scheduled_at && (
                  <div className="flex items-center gap-1.5 text-xs text-muted-foreground mb-4">
                    <Clock className="w-3.5 h-3.5" />
                    <span>Scheduled for: {new Date(camp.scheduled_at).toLocaleString()}</span>
                  </div>
                )}

                {/* Funnel Metrics */}
                {camp.status !== "scheduled" && camp.status !== "draft" && (
                  <div className="grid grid-cols-4 gap-2 text-center bg-white/2 rounded-xl p-3.5 mb-4">
                    <div>
                      <p className="text-[10px] text-muted-foreground font-medium uppercase">Target</p>
                      <p className="text-xs font-bold text-white mt-0.5">{camp.total_recipient_count || 0}</p>
                    </div>
                    <div>
                      <p className="text-[10px] text-muted-foreground font-medium uppercase">Sent</p>
                      <p className="text-xs font-bold text-white mt-0.5">{camp.sent_count || 0}</p>
                    </div>
                    <div>
                      <p className="text-[10px] text-muted-foreground font-medium uppercase">Read</p>
                      <p className="text-xs font-bold text-sky-400 mt-0.5">
                        {(camp.sent_count || 0) > 0 ? `${Math.round(((camp.read_count || 0) / (camp.sent_count || 1)) * 100)}%` : "0%"}
                      </p>
                    </div>
                    <div>
                      <p className="text-[10px] text-muted-foreground font-medium uppercase">Failed</p>
                      <p className="text-xs font-bold text-rose-400 mt-0.5">{camp.failed_count || 0}</p>
                    </div>
                  </div>
                )}
              </div>

              <div className="flex items-center justify-between border-t border-white/5 pt-4 mt-2">
                <button
                  onClick={() => handleCloneCampaign(camp.id)}
                  className="flex items-center gap-1 text-[10px] font-semibold text-muted-foreground hover:text-white transition-colors"
                >
                  <Copy className="w-3.5 h-3.5" /> Clone
                </button>
                <Link
                  href={`/campaigns/${camp.id}`}
                  className="flex items-center gap-1 text-[10px] font-bold text-primary hover:underline"
                >
                  Details <ChevronRight className="w-3.5 h-3.5" />
                </Link>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Campaign Creation Wizard Dialog */}
      {isWizardOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div className="fixed inset-0 bg-slate-950/65 backdrop-blur-sm" onClick={() => setIsWizardOpen(false)} />
          <div className="glass-panel w-full max-w-xl rounded-2xl border border-white/10 p-6 z-10 shadow-2xl relative">
            <button
              onClick={() => setIsWizardOpen(false)}
              className="absolute right-4 top-4 text-muted-foreground hover:text-white"
            >
              <X className="w-5 h-5" />
            </button>

            {/* Stepper Header */}
            <div className="flex items-center justify-between border-b border-white/5 pb-4 mb-5">
              <h2 className="text-sm font-bold text-white">Create New Broadcast</h2>
              <div className="text-xs font-bold text-primary">Step {step} of 4</div>
            </div>

            {/* STEP 1: General Details */}
            {step === 1 && (
              <div className="space-y-4">
                <div>
                  <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Campaign Name</label>
                  <input
                    type="text"
                    required
                    placeholder="e.g. Summer Discount Blast"
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                  />
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Campaign Type</label>
                    <select
                      value={type}
                      onChange={(e) => setType(e.target.value)}
                      className="w-full px-3.5 py-2.5 bg-slate-900 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                    >
                      <option value="promotional">Promotional</option>
                      <option value="transactional">Transactional</option>
                      <option value="reminder">Reminder</option>
                      <option value="survey">Survey</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-xs font-semibold text-muted-foreground mb-1.5">WhatsApp Account</label>
                    <select
                      value={selectedAccount}
                      onChange={(e) => setSelectedAccount(e.target.value)}
                      className="w-full px-3.5 py-2.5 bg-slate-900 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                    >
                      {waAccounts.map(a => (
                        <option key={a.id} value={a.id}>{a.display_name}</option>
                      ))}
                    </select>
                  </div>
                </div>
              </div>
            )}

            {/* STEP 2: Audience Setup */}
            {step === 2 && (
              <div className="space-y-4">
                <div>
                  <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Target Audience Selection</label>
                  <div className="grid grid-cols-2 gap-3 mb-4">
                    <button
                      type="button"
                      onClick={() => setTargetMode("segment")}
                      className={`p-3 rounded-xl border text-center transition-all ${
                        targetMode === "segment" 
                          ? "bg-primary/10 border-primary text-primary" 
                          : "bg-white/2 border-white/10 text-muted-foreground hover:text-white"
                      }`}
                    >
                      <span className="block text-xs font-bold">Filter by Segment</span>
                    </button>
                    <button
                      type="button"
                      onClick={() => setTargetMode("all_contacts")}
                      className={`p-3 rounded-xl border text-center transition-all ${
                        targetMode === "all_contacts" 
                          ? "bg-primary/10 border-primary text-primary" 
                          : "bg-white/2 border-white/10 text-muted-foreground hover:text-white"
                      }`}
                    >
                      <span className="block text-xs font-bold">Broadcast to All Contacts</span>
                    </button>
                  </div>
                </div>

                {targetMode === "segment" && (
                  <div>
                    <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Audience Segment ID</label>
                    <input
                      type="text"
                      placeholder="e.g. uuid-of-segment"
                      value={selectedSegmentId}
                      onChange={(e) => setSelectedSegmentId(e.target.value)}
                      className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                    />
                  </div>
                )}
              </div>
            )}

            {/* STEP 3: Template & Content */}
            {step === 3 && (
              <div className="space-y-4">
                <div>
                  <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Message Composing Method</label>
                  <div className="grid grid-cols-2 gap-3 mb-4">
                    <button
                      type="button"
                      onClick={() => setMessageMode("template")}
                      className={`p-3 rounded-xl border text-center transition-all ${
                        messageMode === "template" 
                          ? "bg-primary/10 border-primary text-primary" 
                          : "bg-white/2 border-white/10 text-muted-foreground hover:text-white"
                      }`}
                    >
                      <span className="block text-xs font-bold">Meta Approved Template</span>
                    </button>
                    <button
                      type="button"
                      onClick={() => setMessageMode("text")}
                      className={`p-3 rounded-xl border text-center transition-all ${
                        messageMode === "text" 
                          ? "bg-primary/10 border-primary text-primary" 
                          : "bg-white/2 border-white/10 text-muted-foreground hover:text-white"
                      }`}
                    >
                      <span className="block text-xs font-bold">Custom Session Text</span>
                    </button>
                  </div>
                </div>

                {messageMode === "template" ? (
                  <div className="space-y-3">
                    <div>
                      <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Select Template</label>
                      <select
                        value={selectedTemplateId}
                        onChange={(e) => handleTemplateChange(e.target.value)}
                        className="w-full px-3.5 py-2.5 bg-slate-900 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                      >
                        <option value="">Choose a template...</option>
                        {templates.map(t => (
                          <option key={t.id} value={t.id}>{t.name} ({t.category})</option>
                        ))}
                      </select>
                    </div>

                    {/* Variable mappings */}
                    {Object.keys(templateVariables).length > 0 && (
                      <div className="bg-white/2 border border-white/5 rounded-xl p-4 space-y-3">
                        <p className="text-[11px] font-bold text-muted-foreground uppercase mb-1">Variable Mappings</p>
                        {Object.keys(templateVariables).map((key) => (
                          <div key={key} className="flex items-center gap-2">
                            <span className="text-xs font-bold text-primary font-mono w-10">{"{{" + key + "}}"}</span>
                            <input
                              type="text"
                              placeholder="Insert mapping value..."
                              value={templateVariables[key]}
                              onChange={(e) => setTemplateVariables({ ...templateVariables, [key]: e.target.value })}
                              className="flex-1 px-3 py-1.5 bg-white/5 border border-white/10 rounded-lg text-xs focus:outline-none focus:ring-1 focus:ring-primary text-white"
                            />
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                ) : (
                  <div>
                    <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Custom Message Body</label>
                    <textarea
                      rows={4}
                      placeholder="Write your custom session message..."
                      value={messageBody}
                      onChange={(e) => setMessageBody(e.target.value)}
                      className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                    />
                  </div>
                )}
              </div>
            )}

            {/* STEP 4: Schedule Settings */}
            {step === 4 && (
              <div className="space-y-4">
                <div>
                  <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Broadcasting Delivery Timing</label>
                  <div className="grid grid-cols-2 gap-3 mb-4">
                    <button
                      type="button"
                      onClick={() => setScheduleMode("immediate")}
                      className={`p-3 rounded-xl border text-center transition-all ${
                        scheduleMode === "immediate" 
                          ? "bg-primary/10 border-primary text-primary" 
                          : "bg-white/2 border-white/10 text-muted-foreground hover:text-white"
                      }`}
                    >
                      <span className="block text-xs font-bold">Send Immediately</span>
                    </button>
                    <button
                      type="button"
                      onClick={() => setScheduleMode("scheduled")}
                      className={`p-3 rounded-xl border text-center transition-all ${
                        scheduleMode === "scheduled" 
                          ? "bg-primary/10 border-primary text-primary" 
                          : "bg-white/2 border-white/10 text-muted-foreground hover:text-white"
                      }`}
                    >
                      <span className="block text-xs font-bold">Schedule for Later</span>
                    </button>
                  </div>
                </div>

                {scheduleMode === "scheduled" && (
                  <div>
                    <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Scheduled Launch Date & Time</label>
                    <input
                      type="datetime-local"
                      value={scheduledDate}
                      onChange={(e) => setScheduledDate(e.target.value)}
                      className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                    />
                  </div>
                )}
              </div>
            )}

            {/* Wizard Buttons */}
            <div className="flex items-center justify-between border-t border-white/5 pt-4 mt-6">
              <button
                type="button"
                disabled={step === 1 || submitting}
                onClick={() => setStep(step - 1)}
                className="px-4 py-2 text-xs font-semibold rounded-xl bg-white/5 border border-white/10 text-white hover:bg-white/10 disabled:opacity-40"
              >
                Back
              </button>

              {step < 4 ? (
                <button
                  type="button"
                  onClick={() => setStep(step + 1)}
                  className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 flex items-center gap-1.5"
                >
                  Continue <ChevronRight className="w-4 h-4" />
                </button>
              ) : (
                <button
                  type="button"
                  disabled={submitting}
                  onClick={handleLaunchCampaign}
                  className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 flex items-center gap-1.5 disabled:opacity-50"
                >
                  {submitting ? <div className="w-4 h-4 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin" /> : "Launch Broadcast"}
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
