"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useAuthStore } from "@/store/authStore";
import { MessageSquare, Building, ArrowRight, Globe, MapPin } from "lucide-react";
import toast from "react-hot-toast";

export default function Onboarding() {
  const router = useRouter();
  const { setOrganization, setTokens, setUser, user } = useAuthStore();

  const [name, setName] = useState("");
  const [website, setWebsite] = useState("");
  const [industry, setIndustry] = useState("");
  const [country, setCountry] = useState("");
  const [timezone, setTimezone] = useState(Intl.DateTimeFormat().resolvedOptions().timeZone);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setError("");

    try {
      const { api } = await import("@/lib/api");

      const res = await api.post("/organizations", {
        name,
        website: website || undefined,
        industry: industry || undefined,
        country: country || undefined,
        timezone: timezone || undefined,
      });

      const org = res.data.data.organization || res.data.data;

      // Re-login to get updated JWT with org_id
      try {
        const refreshToken = (await import("js-cookie")).default.get("refresh_token");
        if (refreshToken) {
          const refreshRes = await api.post("/auth/refresh", { refreshToken });
          const { accessToken: newAccess, newRefreshToken } = refreshRes.data.data;
          setTokens(newAccess, newRefreshToken || refreshToken);
        }
      } catch (refreshErr) {
        // Token refresh failed, but org is created — user can re-login
        console.warn("Token refresh failed after org creation", refreshErr);
      }

      setOrganization(org);
      toast.success("Organization created! Welcome to WhatsUp.");
      router.push("/dashboard");
    } catch (err: any) {
      const errorMsg =
        err.response?.data?.error?.message ||
        err.response?.data?.error ||
        "Failed to create organization. Please try again.";
      const msg = typeof errorMsg === "string" ? errorMsg : "Failed to create organization.";
      setError(msg);
      toast.error(msg);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="relative min-h-screen bg-background flex items-center justify-center p-6 overflow-hidden">
      <div className="absolute top-[50%] left-[50%] transform translate-x-[-50%] translate-y-[-50%] w-[600px] h-[600px] rounded-full bg-primary/5 blur-[120px] pointer-events-none"></div>

      <div className="w-full max-w-md z-10">
        {/* Logo Header */}
        <div className="flex flex-col items-center mb-8">
          <div className="w-12 h-12 rounded-2xl bg-gradient-to-tr from-primary to-emerald-500 flex items-center justify-center shadow-lg shadow-primary/20 mb-4">
            <MessageSquare className="w-6 h-6 text-primary-foreground" />
          </div>
          <h2 className="text-2xl font-bold tracking-tight">Set up your organization</h2>
          <p className="text-muted-foreground text-sm mt-1">
            {user ? `Welcome, ${user.full_name}!` : "One last step before you start broadcasting"}
          </p>
        </div>

        {/* Form Container */}
        <div className="glass-panel p-8 rounded-2xl border border-white/10 shadow-2xl">
          <form onSubmit={handleSubmit} className="space-y-5">
            <div>
              <label htmlFor="orgName" className="block text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
                Organization Name *
              </label>
              <div className="relative">
                <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                  <Building className="h-5 w-5 text-muted-foreground/60" />
                </div>
                <input
                  id="orgName"
                  type="text"
                  required
                  placeholder="Acme Corp"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="block w-full pl-11 pr-4 py-3 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all placeholder-white/20 text-sm text-white"
                />
              </div>
            </div>

            <div>
              <label htmlFor="website" className="block text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
                Website
              </label>
              <div className="relative">
                <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                  <Globe className="h-5 w-5 text-muted-foreground/60" />
                </div>
                <input
                  id="website"
                  type="url"
                  placeholder="https://acme.com"
                  value={website}
                  onChange={(e) => setWebsite(e.target.value)}
                  className="block w-full pl-11 pr-4 py-3 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all placeholder-white/20 text-sm text-white"
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label htmlFor="industry" className="block text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
                  Industry
                </label>
                <select
                  id="industry"
                  value={industry}
                  onChange={(e) => setIndustry(e.target.value)}
                  className="w-full px-3.5 py-3 bg-slate-900 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary text-sm text-white"
                >
                  <option value="">Select...</option>
                  <option value="ecommerce">E-Commerce</option>
                  <option value="saas">SaaS</option>
                  <option value="fintech">FinTech</option>
                  <option value="healthcare">Healthcare</option>
                  <option value="education">Education</option>
                  <option value="real_estate">Real Estate</option>
                  <option value="travel">Travel</option>
                  <option value="other">Other</option>
                </select>
              </div>
              <div>
                <label htmlFor="country" className="block text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
                  Country
                </label>
                <div className="relative">
                  <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                    <MapPin className="h-4 w-4 text-muted-foreground/60" />
                  </div>
                  <input
                    id="country"
                    type="text"
                    placeholder="India"
                    value={country}
                    onChange={(e) => setCountry(e.target.value)}
                    className="block w-full pl-10 pr-4 py-3 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all placeholder-white/20 text-sm text-white"
                  />
                </div>
              </div>
            </div>

            {error && (
              <div className="p-3 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-semibold">
                {error}
              </div>
            )}

            <button
              type="submit"
              disabled={isLoading}
              className="w-full py-3.5 px-4 bg-primary text-primary-foreground font-semibold rounded-xl hover:bg-primary/95 transition-all hover-scale shadow-lg shadow-primary/25 disabled:opacity-50 disabled:pointer-events-none flex items-center justify-center gap-2 text-sm"
            >
              {isLoading ? (
                <div className="w-5 h-5 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin"></div>
              ) : (
                <>Create Organization <ArrowRight className="w-4 h-4" /></>
              )}
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}
