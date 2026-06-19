"use client";

import { useEffect, useState } from "react";
import {
  Smartphone,
  Plus,
  RefreshCw,
  Trash2,
  Lock,
  X,
  Globe,
  AlertCircle
} from "lucide-react";
import toast from "react-hot-toast";

interface WaAccount {
  id: string;
  display_name: string;
  phone_number: string;
  status: string;
  type: string;
  quality_rating: string | null;
  total_msgs_sent: number;
  connected_at: string | null;
}

export default function WhatsAppAccountsManager() {
  const [accounts, setAccounts] = useState<WaAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [isOpen, setIsOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  // Form State
  const [displayName, setDisplayName] = useState("");
  const [phoneNumber, setPhoneNumber] = useState("");
  const [phoneNumberId, setPhoneNumberId] = useState("");
  const [wabaId, setWabaId] = useState("");
  const [accessToken, setAccessToken] = useState("");

  const fetchAccounts = async () => {
    setLoading(true);
    setError("");
    try {
      const { api } = await import("@/lib/api");
      const res = await api.get("/whatsapp/accounts");
      setAccounts(res.data.data.accounts || []);
    } catch (e: any) {
      const msg = e.response?.data?.error?.message || "Failed to load WhatsApp accounts";
      setError(typeof msg === "string" ? msg : "Failed to load accounts");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchAccounts();
  }, []);

  const handleLinkAccount = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    try {
      const { api } = await import("@/lib/api");
      await api.post("/whatsapp/accounts", {
        display_name: displayName,
        phone_number: phoneNumber,
        phone_number_id: phoneNumberId,
        waba_id: wabaId,
        access_token: accessToken,
      });

      toast.success("WhatsApp Account connected successfully!");
      setIsOpen(false);
      resetForm();
      fetchAccounts();
    } catch (e: any) {
      const msg = e.response?.data?.error?.message || "Failed to link WhatsApp account";
      toast.error(typeof msg === "string" ? msg : "Failed to link account");
    } finally {
      setSubmitting(false);
    }
  };

  const handleSyncMeta = async (id: string) => {
    try {
      const { api } = await import("@/lib/api");
      await api.post(`/whatsapp/accounts/${id}/sync`);
      toast.success("Account profile synced with Meta Cloud!");
      fetchAccounts();
    } catch (e: any) {
      const msg = e.response?.data?.error?.message || "Sync failed";
      toast.error(typeof msg === "string" ? msg : "Sync failed");
    }
  };

  const handleDisconnect = async (id: string) => {
    if (!confirm("Are you sure you want to disconnect this number? Bulk broadcasts will stop.")) return;

    try {
      const { api } = await import("@/lib/api");
      await api.delete(`/whatsapp/accounts/${id}`);
      toast.success("Account disconnected");
      fetchAccounts();
    } catch (e: any) {
      const msg = e.response?.data?.error?.message || "Failed to disconnect";
      toast.error(typeof msg === "string" ? msg : "Failed to disconnect");
    }
  };

  const resetForm = () => {
    setDisplayName("");
    setPhoneNumber("");
    setPhoneNumberId("");
    setWabaId("");
    setAccessToken("");
  };

  return (
    <div className="space-y-6">
      {/* Title Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-white/5 pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
            <Smartphone className="w-6 h-6 text-primary" /> WhatsApp Accounts
          </h1>
          <p className="text-muted-foreground text-sm">Link official Meta business phone numbers and track API connection metrics</p>
        </div>

        <button
          onClick={() => setIsOpen(true)}
          className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 hover-scale flex items-center gap-1.5"
        >
          <Plus className="w-4 h-4" /> Link Number
        </button>
      </div>

      {/* Webhook Configuration Guide Banner */}
      <div className="p-4 rounded-2xl bg-white/2 border border-white/15 flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
        <div className="flex items-start gap-3">
          <Globe className="w-5 h-5 text-primary flex-shrink-0 mt-0.5" />
          <div className="text-xs">
            <span className="font-bold text-white block">Meta Developer Webhook Settings</span>
            <span className="text-muted-foreground mt-0.5 block">Copy these details into your Meta App dashboard to enable message delivery updates.</span>
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-[10px] font-mono bg-slate-950/40 p-3 rounded-xl border border-white/5 w-full md:w-auto">
          <div>
            <span className="text-muted-foreground block font-sans">CALLBACK URL:</span>
            <span className="text-primary font-bold">http://localhost:8081/api/v1/webhooks/whatsapp</span>
          </div>
          <div>
            <span className="text-muted-foreground block font-sans">VERIFY TOKEN:</span>
            <span className="text-white font-bold">whatsapp_verify_token_secure</span>
          </div>
        </div>
      </div>

      {/* Grid of Accounts */}
      {loading ? (
        <div className="flex justify-center items-center py-20">
          <div className="w-8 h-8 border-4 border-primary/20 border-t-primary rounded-full animate-spin" />
        </div>
      ) : error ? (
        <div className="glass-panel p-10 rounded-2xl border border-white/5 text-center space-y-4">
          <AlertCircle className="w-8 h-8 text-rose-400 mx-auto" />
          <p className="text-muted-foreground text-sm">{error}</p>
          <button onClick={fetchAccounts} className="px-4 py-2 text-xs font-semibold rounded-xl bg-white/5 border border-white/10 text-white hover:bg-white/10">
            <RefreshCw className="w-3.5 h-3.5 inline mr-1.5" /> Retry
          </button>
        </div>
      ) : accounts.length === 0 ? (
        <div className="glass-panel p-10 rounded-2xl border border-white/5 text-center text-muted-foreground text-sm">
          No numbers linked yet. Add a WhatsApp Business number credentials to start broadcasting.
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {accounts.map((acc) => (
            <div key={acc.id} className="glass-panel p-5 rounded-2xl border border-white/5 flex flex-col justify-between hover-scale group">
              <div>
                <div className="flex items-center justify-between mb-4">
                  <span className={`inline-flex items-center gap-1 text-[9px] font-bold px-2 py-0.5 rounded-full uppercase ${
                    acc.status === "connected" ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20" :
                    "bg-rose-500/10 text-rose-400 border border-rose-500/20"
                  }`}>
                    {acc.status}
                  </span>
                  <div className="w-9 h-9 rounded-full bg-gradient-to-tr from-primary to-accent-gradient flex items-center justify-center font-bold text-sm text-primary-foreground uppercase shadow shadow-primary/20">
                    {(acc.display_name || "WA").slice(0, 2)}
                  </div>
                </div>

                <h3 className="text-sm font-bold text-white mb-1">{acc.display_name}</h3>
                <p className="text-xs text-muted-foreground font-semibold font-mono mb-4">{acc.phone_number || "No phone configured"}</p>

                <div className="space-y-2 text-xs text-muted-foreground bg-white/2 rounded-xl p-3.5 mb-4 font-mono text-[10px]">
                  <div className="flex items-center justify-between">
                    <span className="font-sans">Messages Sent:</span>
                    <span className="font-bold text-white">{acc.total_msgs_sent || 0}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="font-sans">Quality Rating:</span>
                    <span className="font-bold text-white">{acc.quality_rating || "N/A"}</span>
                  </div>
                </div>
              </div>

              <div className="flex items-center justify-between border-t border-white/5 pt-4 mt-2">
                <button
                  onClick={() => handleSyncMeta(acc.id)}
                  className="flex items-center gap-1.5 text-xs font-semibold px-3 py-1.5 rounded-lg border border-white/10 text-white hover:bg-white/5 transition-all"
                >
                  <RefreshCw className="w-3.5 h-3.5" /> Re-sync
                </button>
                <button
                  onClick={() => handleDisconnect(acc.id)}
                  className="px-3 py-1.5 text-xs font-semibold rounded-lg border border-rose-500/20 bg-rose-500/5 text-rose-400 hover:bg-rose-500/10 transition-all flex items-center gap-1"
                >
                  <Trash2 className="w-3.5 h-3.5" /> Disconnect
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Modal: Link Number */}
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
            <h2 className="text-lg font-bold text-white mb-4">Link Meta Phone Number</h2>

            <form onSubmit={handleLinkAccount} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Account Label Name</label>
                <input
                  type="text"
                  required
                  placeholder="e.g. Acme Support Team"
                  value={displayName}
                  onChange={(e) => setDisplayName(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Phone Number (with Country Code)</label>
                <input
                  type="text"
                  required
                  placeholder="+919876543210"
                  value={phoneNumber}
                  onChange={(e) => setPhoneNumber(e.target.value)}
                  className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Phone Number ID</label>
                  <input
                    type="text"
                    required
                    placeholder="10928392..."
                    value={phoneNumberId}
                    onChange={(e) => setPhoneNumberId(e.target.value)}
                    className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white font-mono"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-muted-foreground mb-1.5">WABA ID</label>
                  <input
                    type="text"
                    required
                    placeholder="90812309..."
                    value={wabaId}
                    onChange={(e) => setWabaId(e.target.value)}
                    className="w-full px-3.5 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white font-mono"
                  />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-muted-foreground mb-1.5">Meta Permanent Access Token</label>
                <div className="relative">
                  <input
                    type="password"
                    required
                    placeholder="EAAGy..."
                    value={accessToken}
                    onChange={(e) => setAccessToken(e.target.value)}
                    className="w-full pl-9 pr-4 py-2.5 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white font-mono"
                  />
                  <Lock className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground/60" />
                </div>
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
                  disabled={submitting}
                  className="px-4 py-2 text-xs font-semibold rounded-xl bg-primary text-primary-foreground hover:bg-primary/95 disabled:opacity-50 flex items-center gap-1.5"
                >
                  {submitting ? (
                    <div className="w-4 h-4 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin" />
                  ) : (
                    "Link Account"
                  )}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
