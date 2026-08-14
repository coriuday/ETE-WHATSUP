"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useAuthStore } from "@/store/authStore";
import toast from "react-hot-toast";
import { AuthSplit } from "@/components/auth/auth-split";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export default function Onboarding() {
  const router = useRouter();
  const { setOrganization, setTokens, user } = useAuthStore();

  const [name, setName] = useState("");
  const [website, setWebsite] = useState("");
  const [industry, setIndustry] = useState("");
  const [country, setCountry] = useState("");
  const [timezone] = useState(Intl.DateTimeFormat().resolvedOptions().timeZone);
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

      try {
        const refreshToken = (await import("js-cookie")).default.get("refresh_token");
        if (refreshToken) {
          const refreshRes = await api.post("/auth/refresh", { refreshToken });
          const { accessToken: newAccess, newRefreshToken } = refreshRes.data.data;
          setTokens(newAccess, newRefreshToken || refreshToken);
        }
      } catch (refreshErr) {
        console.warn("Token refresh failed after org creation", refreshErr);
      }

      setOrganization(org);
      toast.success("Organization created! Welcome to WhatsUp.");
      router.push("/dashboard");
    } catch (err: unknown) {
      const axiosErr = err as { response?: { data?: { error?: { message?: string } | string } } };
      const errorMsg =
        (typeof axiosErr.response?.data?.error === "object"
          ? axiosErr.response?.data?.error?.message
          : axiosErr.response?.data?.error) || "Failed to create organization. Please try again.";
      const msg = typeof errorMsg === "string" ? errorMsg : "Failed to create organization.";
      setError(msg);
      toast.error(msg);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <AuthSplit
      title="Set up your organization"
      description={user ? `Welcome, ${user.fullName}.` : "One last step before inbox and campaigns."}
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="orgName">Organization name</Label>
          <Input id="orgName" required placeholder="Acme Corp" value={name} onChange={(e) => setName(e.target.value)} />
        </div>
        <div className="space-y-2">
          <Label htmlFor="website">Website</Label>
          <Input id="website" type="url" placeholder="https://acme.com" value={website} onChange={(e) => setWebsite(e.target.value)} />
        </div>
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <Label htmlFor="industry">Industry</Label>
            <select
              id="industry"
              value={industry}
              onChange={(e) => setIndustry(e.target.value)}
              className="flex h-9 w-full rounded-lg border border-input bg-background px-3 text-sm"
            >
              <option value="">Select…</option>
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
          <div className="space-y-2">
            <Label htmlFor="country">Country</Label>
            <Input id="country" placeholder="India" value={country} onChange={(e) => setCountry(e.target.value)} />
          </div>
        </div>
        {error && <p className="text-xs text-destructive">{error}</p>}
        <Button type="submit" className="w-full" disabled={isLoading}>
          {isLoading ? "Creating…" : "Create organization"}
        </Button>
      </form>
    </AuthSplit>
  );
}
