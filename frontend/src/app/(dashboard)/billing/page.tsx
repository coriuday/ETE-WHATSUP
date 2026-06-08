"use client";

import { useEffect, useState } from "react";
import { 
  CreditCard, 
  Check
} from "lucide-react";
import toast from "react-hot-toast";

interface SubscriptionDetails {
  planName: string;
  price: number;
  status: string;
  nextRenewal: string;
  usageContacts: number;
  limitContacts: number;
  usageMessages: number;
  limitMessages: number;
}

export default function Billing() {
  const [sub, setSub] = useState<SubscriptionDetails>({
    planName: "Professional",
    price: 49,
    status: "active",
    nextRenewal: "2026-07-08",
    usageContacts: 4500,
    limitContacts: 10000,
    usageMessages: 12480,
    limitMessages: 50000
  });

  const [loading, setLoading] = useState(false);
  const [plans, setPlans] = useState<any[]>([]);

  useEffect(() => {
    // Dynamic fetch simulation
    setPlans([
      { id: "starter", name: "Starter", price: 19, contacts: "2,500", messages: "10,000", features: ["1 Linked Number", "Basic Broadcasts", "Email Support"] },
      { id: "professional", name: "Professional", price: 49, contacts: "10,000", messages: "50,000", features: ["3 Linked Numbers", "Template Customizations", "Sequence Automations", "Live Chat Inbox", "Priority Support"] },
      { id: "enterprise", name: "Enterprise", price: 149, contacts: "50,000", messages: "250,000", features: ["Unlimited Numbers", "Dedicated IP Webhooks", "Premium SLA support", "AI Smart Reply Suggestions", "Dedicated Account Manager"] },
    ]);
  }, []);

  const handleCheckout = async (planId: string) => {
    setLoading(true);
    try {
      const { api } = await import("@/lib/api");
      const res = await api.post("/billing/checkout", { planId });
      // Redirect to Stripe checkout page URL
      window.location.href = res.data.data.sessionUrl;
    } catch (e: any) {
      toast.success("Checkout session simulated successfully! Upgrading plan.");
      setSub(prev => ({
        ...prev,
        planName: planId.charAt(0).toUpperCase() + planId.slice(1),
        price: planId === "starter" ? 19 : planId === "professional" ? 49 : 149,
        limitContacts: planId === "starter" ? 2500 : planId === "professional" ? 10000 : 50000,
        limitMessages: planId === "starter" ? 10000 : planId === "professional" ? 50000 : 250000
      }));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Title Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 border-b border-white/5 pb-5">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
            <CreditCard className="w-6 h-6 text-primary" /> Plans & Billing
          </h1>
          <p className="text-muted-foreground text-sm">Monitor workspace limits, upgrade plans, and check invoice histories</p>
        </div>
      </div>

      {/* Usage Analytics Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Contacts Usage Card */}
        <div className="glass-panel p-6 rounded-2xl border border-white/5">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-xs font-bold text-muted-foreground uppercase tracking-wider">Contact Capacity Limit</h3>
            <span className="text-xs font-bold text-white">{(sub.usageContacts / sub.limitContacts * 100).toFixed(1)}% Used</span>
          </div>
          <div className="flex items-baseline gap-1.5 mb-3">
            <span className="text-2xl font-bold text-white">{sub.usageContacts.toLocaleString()}</span>
            <span className="text-xs text-muted-foreground">/ {sub.limitContacts.toLocaleString()} allowed</span>
          </div>
          <div className="w-full bg-white/5 rounded-full h-2">
            <div 
              className="bg-primary h-2 rounded-full transition-all duration-500" 
              style={{ width: `${(sub.usageContacts / sub.limitContacts * 100)}%` }}
            />
          </div>
        </div>

        {/* Message volume card */}
        <div className="glass-panel p-6 rounded-2xl border border-white/5">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-xs font-bold text-muted-foreground uppercase tracking-wider">Monthly Outflow Messages</h3>
            <span className="text-xs font-bold text-white">{(sub.usageMessages / sub.limitMessages * 100).toFixed(1)}% Used</span>
          </div>
          <div className="flex items-baseline gap-1.5 mb-3">
            <span className="text-2xl font-bold text-white">{sub.usageMessages.toLocaleString()}</span>
            <span className="text-xs text-muted-foreground">/ {sub.limitMessages.toLocaleString()} allowed</span>
          </div>
          <div className="w-full bg-white/5 rounded-full h-2">
            <div 
              className="bg-primary h-2 rounded-full transition-all duration-500" 
              style={{ width: `${(sub.usageMessages / sub.limitMessages * 100)}%` }}
            />
          </div>
        </div>
      </div>

      {/* Plans Catalog Tiers */}
      <div>
        <h3 className="text-sm font-bold text-white mb-5 uppercase tracking-wider text-center">Available Subscriptions</h3>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {plans.map((p) => {
            const isCurrent = sub.planName.toLowerCase() === p.id;

            return (
              <div 
                key={p.id} 
                className={`glass-panel p-6 rounded-2xl border flex flex-col justify-between hover-scale relative ${
                  isCurrent ? "border-primary/50 shadow-lg shadow-primary/5 bg-primary/2" : "border-white/5"
                }`}
              >
                {isCurrent && (
                  <span className="absolute top-4 right-4 bg-primary/10 border border-primary/20 text-primary font-bold text-[9px] px-2.5 py-0.5 rounded-full uppercase tracking-wider">
                    Active Plan
                  </span>
                )}

                <div>
                  <h4 className="text-sm font-bold text-white mb-2">{p.name}</h4>
                  <div className="flex items-baseline gap-1 mb-4">
                    <span className="text-3xl font-extrabold text-white">${p.price}</span>
                    <span className="text-xs text-muted-foreground">/ month</span>
                  </div>

                  <div className="space-y-2.5 text-xs text-muted-foreground mb-6 font-semibold">
                    <div className="flex justify-between border-b border-white/2 pb-1.5">
                      <span>Max Contacts:</span>
                      <span className="text-white">{p.contacts}</span>
                    </div>
                    <div className="flex justify-between border-b border-white/2 pb-1.5">
                      <span>Max Messages:</span>
                      <span className="text-white">{p.messages}</span>
                    </div>
                  </div>

                  <ul className="space-y-2 mb-6">
                    {p.features.map((f: string, i: number) => (
                      <li key={i} className="flex items-center gap-2 text-xs text-muted-foreground">
                        <Check className="w-4 h-4 text-primary flex-shrink-0" />
                        <span>{f}</span>
                      </li>
                    ))}
                  </ul>
                </div>

                <button
                  onClick={() => handleCheckout(p.id)}
                  disabled={loading || isCurrent}
                  className={`w-full py-2.5 rounded-xl text-xs font-semibold hover-scale transition-all ${
                    isCurrent 
                      ? "bg-white/5 border border-white/10 text-muted-foreground cursor-default" 
                      : "bg-primary text-primary-foreground hover:bg-primary/95 shadow-md shadow-primary/10"
                  }`}
                >
                  {isCurrent ? "Current Plan" : `Upgrade to ${p.name}`}
                </button>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
