"use client";

import { useState } from "react";
import Link from "next/link";
import { MessageSquare, Mail, ArrowRight, ArrowLeft, CheckCircle2 } from "lucide-react";
import toast from "react-hot-toast";

export default function ForgotPassword() {
  const [email, setEmail] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isSent, setIsSent] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      const { api } = await import("@/lib/api");
      await api.post("/auth/forgot-password", { email });
      setIsSent(true);
      toast.success("Reset link sent successfully!");
    } catch (err: any) {
      const errorMsg = err.response?.data?.error || "Failed to send reset link. Please verify your email.";
      toast.error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="relative min-h-screen bg-background flex items-center justify-center p-6 overflow-hidden">
      <div className="absolute top-[50%] left-[50%] transform translate-x-[-50%] translate-y-[-50%] w-[600px] h-[600px] rounded-full bg-primary/5 blur-[120px] pointer-events-none"></div>

      <div className="w-full max-w-md z-10">
        <div className="flex flex-col items-center mb-8">
          <div className="w-12 h-12 rounded-2xl bg-gradient-to-tr from-primary to-emerald-500 flex items-center justify-center shadow-lg shadow-primary/20 mb-4">
            <MessageSquare className="w-6 h-6 text-primary-foreground" />
          </div>
          <h2 className="text-2xl font-bold tracking-tight">Reset password</h2>
          <p className="text-muted-foreground text-sm mt-1">We will send you a link to reset your password</p>
        </div>

        <div className="glass-panel p-8 rounded-2xl border border-white/10 shadow-2xl">
          {!isSent ? (
            <form onSubmit={handleSubmit} className="space-y-6">
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

              <button
                type="submit"
                disabled={isLoading}
                className="w-full py-3.5 px-4 bg-primary text-primary-foreground font-semibold rounded-xl hover:bg-primary/95 transition-all hover-scale shadow-lg shadow-primary/25 disabled:opacity-50 disabled:pointer-events-none flex items-center justify-center gap-2 text-sm"
              >
                {isLoading ? (
                  <div className="w-5 h-5 border-2 border-primary-foreground border-t-transparent rounded-full animate-spin"></div>
                ) : (
                  <>Send Reset Link <ArrowRight className="w-4 h-4" /></>
                )}
              </button>
            </form>
          ) : (
            <div className="text-center space-y-4">
              <div className="w-12 h-12 rounded-full bg-primary/10 border border-primary/20 flex items-center justify-center mx-auto mb-2 text-primary">
                <CheckCircle2 className="w-6 h-6" />
              </div>
              <h3 className="font-bold text-lg">Check your inbox</h3>
              <p className="text-muted-foreground text-sm leading-relaxed">
                If the email is registered, we have sent password reset instructions to <span className="text-white font-medium">{email}</span>.
              </p>
            </div>
          )}

          <div className="mt-6 text-center">
            <Link href="/login" className="inline-flex items-center gap-1.5 text-xs font-semibold text-primary hover:underline">
              <ArrowLeft className="w-3.5 h-3.5" /> Back to Sign In
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}
