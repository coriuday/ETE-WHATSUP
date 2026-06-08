"use client";

import { useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { MessageSquare, CheckCircle, XCircle, Loader2, ArrowRight } from "lucide-react";

export default function VerifyEmail() {
  const searchParams = useSearchParams();
  const token = searchParams.get("token");

  const [status, setStatus] = useState<"loading" | "success" | "error">("loading");
  const [message, setMessage] = useState("Verifying your email address...");

  useEffect(() => {
    if (!token) {
      setStatus("error");
      setMessage("Verification token is missing.");
      return;
    }

    const verify = async () => {
      try {
        const { api } = await import("@/lib/api");
        await api.get(`/auth/verify-email?token=${token}`);
        setStatus("success");
        setMessage("Your email has been verified successfully! You can now log in.");
      } catch (err: any) {
        setStatus("error");
        setMessage(err.response?.data?.error || "Email verification failed or the token has expired.");
      }
    };

    verify();
  }, [token]);

  return (
    <div className="relative min-h-screen bg-background flex items-center justify-center p-6 overflow-hidden">
      <div className="absolute top-[50%] left-[50%] transform translate-x-[-50%] translate-y-[-50%] w-[600px] h-[600px] rounded-full bg-primary/5 blur-[120px] pointer-events-none"></div>

      <div className="w-full max-w-md z-10">
        <div className="flex flex-col items-center mb-8">
          <div className="w-12 h-12 rounded-2xl bg-gradient-to-tr from-primary to-emerald-500 flex items-center justify-center shadow-lg shadow-primary/20 mb-4">
            <MessageSquare className="w-6 h-6 text-primary-foreground" />
          </div>
          <h2 className="text-2xl font-bold tracking-tight">Email Verification</h2>
        </div>

        <div className="glass-panel p-8 rounded-2xl border border-white/10 shadow-2xl text-center">
          {status === "loading" && (
            <div className="space-y-4 py-6">
              <Loader2 className="w-12 h-12 text-primary animate-spin mx-auto" />
              <p className="text-muted-foreground text-sm">{message}</p>
            </div>
          )}

          {status === "success" && (
            <div className="space-y-6 py-4">
              <div className="w-16 h-16 rounded-full bg-primary/10 border border-primary/20 flex items-center justify-center mx-auto text-primary">
                <CheckCircle className="w-8 h-8" />
              </div>
              <div className="space-y-2">
                <h3 className="text-lg font-bold">Email Verified</h3>
                <p className="text-muted-foreground text-sm leading-relaxed">{message}</p>
              </div>
              <Link
                href="/login"
                className="w-full py-3.5 px-4 bg-primary text-primary-foreground font-semibold rounded-xl hover:bg-primary/95 transition-all hover-scale shadow-lg shadow-primary/25 flex items-center justify-center gap-2 text-sm"
              >
                Go to Sign In <ArrowRight className="w-4 h-4" />
              </Link>
            </div>
          )}

          {status === "error" && (
            <div className="space-y-6 py-4">
              <div className="w-16 h-16 rounded-full bg-destructive/10 border border-destructive/20 flex items-center justify-center mx-auto text-destructive">
                <XCircle className="w-8 h-8" />
              </div>
              <div className="space-y-2">
                <h3 className="text-lg font-bold">Verification Failed</h3>
                <p className="text-muted-foreground text-sm leading-relaxed">{message}</p>
              </div>
              <Link
                href="/login"
                className="inline-block font-semibold text-primary hover:underline text-sm"
              >
                Back to Sign In
              </Link>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
