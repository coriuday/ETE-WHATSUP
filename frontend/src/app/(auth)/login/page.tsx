"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuthStore } from "@/store/authStore";
import { MessageSquare, Lock, Mail, ArrowRight, ShieldCheck, Eye, EyeOff } from "lucide-react";
import toast from "react-hot-toast";

export default function Login() {
  const router = useRouter();
  const { setTokens, setUser, setOrganization } = useAuthStore();
  
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  
  // 2FA state
  const [requires2Fa, setRequires2Fa] = useState(false);
  const [twoFactorToken, setTwoFactorToken] = useState("");
  const [code, setCode] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      const { api } = await import("@/lib/api");
      
      if (requires2Fa) {
        // Verify 2FA code
        const res = await api.post("/auth/2fa/verify", {
          token: twoFactorToken,
          code,
        });

        const { accessToken, refreshToken, user } = res.data.data;
        setTokens(accessToken, refreshToken);
        setUser(user);
        
        // Fetch organization
        try {
          const orgRes = await api.get("/organizations");
          const orgs = orgRes.data.data.organizations || [];
          if (orgs.length > 0) setOrganization(orgs[0]);
        } catch (e) {
          console.error("Failed loading orgs", e);
        }

        toast.success("Welcome back!");
        router.push("/dashboard");
      } else {
        // Standard email/password login
        const res = await api.post("/auth/login", { email, password });
        
        if (res.data.data.requires2fa) {
          setRequires2Fa(true);
          setTwoFactorToken(res.data.data.token);
          toast.success("Please enter your 2FA verification code");
        } else {
          const { accessToken, refreshToken, user } = res.data.data;
          setTokens(accessToken, refreshToken);
          setUser(user);

          // Fetch organization
          try {
            const orgRes = await api.get("/organizations");
            const orgs = orgRes.data.data.organizations || [];
            if (orgs.length > 0) setOrganization(orgs[0]);
          } catch (e) {
            console.error("Failed loading orgs", e);
          }

          toast.success("Logged in successfully!");
          router.push("/dashboard");
        }
      }
    } catch (err: any) {
      const errorMsg = err.response?.data?.error || "Invalid credentials. Please try again.";
      toast.error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="relative min-h-screen bg-background flex items-center justify-center p-6 overflow-hidden">
      {/* Background radial highlight */}
      <div className="absolute top-[50%] left-[50%] transform translate-x-[-50%] translate-y-[-50%] w-[600px] h-[600px] rounded-full bg-primary/5 blur-[120px] pointer-events-none"></div>

      <div className="w-full max-w-md z-10">
        {/* Logo Header */}
        <div className="flex flex-col items-center mb-8">
          <div className="w-12 h-12 rounded-2xl bg-gradient-to-tr from-primary to-emerald-500 flex items-center justify-center shadow-lg shadow-primary/20 mb-4">
            <MessageSquare className="w-6 h-6 text-primary-foreground" />
          </div>
          <h2 className="text-2xl font-bold tracking-tight">Sign in to WhatsUp</h2>
          <p className="text-muted-foreground text-sm mt-1">Manage bulk campaigns and conversations</p>
        </div>

        {/* Login Form Container */}
        <div className="glass-panel p-8 rounded-2xl border border-white/10 shadow-2xl">
          <form onSubmit={handleSubmit} className="space-y-6">
            {!requires2Fa ? (
              <>
                <div>
                  <label htmlFor="email" className="block text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
                    Email Address
                  </label>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                      <Mail className="h-5 w-5 text-muted-foreground/60" />
                    </div>
                    <input
                      id="email"
                      type="email"
                      required
                      placeholder="you@company.com"
                      value={email}
                      onChange={(e) => setEmail(e.target.value)}
                      className="block w-full pl-11 pr-4 py-3 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all placeholder-white/20 text-sm text-white"
                    />
                  </div>
                </div>

                <div>
                  <div className="flex items-center justify-between mb-2">
                    <label htmlFor="password" className="block text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                      Password
                    </label>
                    <Link href="/forgot-password" className="text-xs font-semibold text-primary hover:underline">
                      Forgot Password?
                    </Link>
                  </div>
                  <div className="relative">
                    <div className="absolute inset-y-0 left-0 pl-3.5 flex items-center pointer-events-none">
                      <Lock className="h-5 w-5 text-muted-foreground/60" />
                    </div>
                    <input
                      id="password"
                      type={showPassword ? "text" : "password"}
                      required
                      placeholder="••••••••"
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      className="block w-full pl-11 pr-12 py-3 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all placeholder-white/20 text-sm text-white"
                    />
                    <button
                      type="button"
                      onClick={() => setShowPassword(!showPassword)}
                      className="absolute inset-y-0 right-0 pr-3.5 flex items-center text-muted-foreground/60 hover:text-white transition-colors"
                    >
                      {showPassword ? <EyeOff className="w-5 h-5" /> : <Eye className="w-5 h-5" />}
                    </button>
                  </div>
                </div>
              </>
            ) : (
              <div>
                <div className="flex items-center gap-2 mb-4 p-3.5 rounded-xl bg-primary/10 border border-primary/20 text-primary text-xs">
                  <ShieldCheck className="w-5 h-5 flex-shrink-0" />
                  <span>Two-Factor Authentication is enabled. Please check your authenticator app for the code.</span>
                </div>
                <label htmlFor="code" className="block text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
                  Authenticator Code
                </label>
                <input
                  id="code"
                  type="text"
                  required
                  placeholder="123456"
                  maxLength={6}
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  className="block w-full px-4 py-3 bg-white/5 border border-white/10 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent transition-all text-center tracking-widest text-lg font-bold text-white placeholder-white/10"
                />
              </div>
            )}

            <button
              type="submit"
              disabled={isLoading}
              className="w-full py-3.5 px-4 bg-primary text-primary-foreground font-semibold rounded-xl hover:bg-primary/95 transition-all hover-scale shadow-lg shadow-primary/25 disabled:opacity-50 disabled:pointer-events-none flex items-center justify-center gap-2 text-sm"
            >
              {isLoading ? (
                <div className="w-5 h-5 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin"></div>
              ) : requires2Fa ? (
                <>Verify 2FA Code <ArrowRight className="w-4 h-4" /></>
              ) : (
                <>Sign In <ArrowRight className="w-4 h-4" /></>
              )}
            </button>
          </form>

          {!requires2Fa && (
            <div className="mt-6 text-center text-xs text-muted-foreground">
              Don&apos;t have an account?{" "}
              <Link href="/register" className="font-semibold text-primary hover:underline">
                Create an account
              </Link>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
