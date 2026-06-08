"use client";

import { useAuthStore } from "@/store/authStore";
import Link from "next/link";
import { MessageSquare, ArrowRight, Shield, Zap, BarChart3 } from "lucide-react";

export default function Home() {
  const { isAuthenticated } = useAuthStore();

  return (
    <div className="relative min-h-screen bg-background flex flex-col justify-between overflow-hidden">
      {/* Background glow effects */}
      <div className="absolute top-[-20%] left-[-10%] w-[50%] h-[60%] rounded-full bg-primary/10 blur-[120px] pointer-events-none"></div>
      <div className="absolute bottom-[-20%] right-[-10%] w-[50%] h-[60%] rounded-full bg-primary/5 blur-[120px] pointer-events-none"></div>

      {/* Header */}
      <header className="px-6 py-6 flex items-center justify-between border-b border-white/5 backdrop-blur-md sticky top-0 z-50">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-primary to-accent-gradient flex items-center justify-center shadow-lg shadow-primary/20">
            <MessageSquare className="w-5 h-5 text-primary-foreground" />
          </div>
          <span className="text-xl font-bold tracking-tight bg-gradient-to-r from-white via-white to-primary/80 bg-clip-text text-transparent">
            WhatsUp
          </span>
        </div>

        <div>
          {isAuthenticated ? (
            <Link
              href="/dashboard"
              className="px-5 py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold hover:bg-primary/95 hover-scale flex items-center gap-2 text-sm"
            >
              Go to Dashboard <ArrowRight className="w-4 h-4" />
            </Link>
          ) : (
            <div className="flex items-center gap-4">
              <Link href="/login" className="text-sm font-semibold hover:text-primary transition-colors">
                Sign In
              </Link>
              <Link
                href="/register"
                className="px-5 py-2.5 rounded-xl bg-primary text-primary-foreground font-semibold hover:bg-primary/95 hover-scale text-sm"
              >
                Get Started
              </Link>
            </div>
          )}
        </div>
      </header>

      {/* Hero Content */}
      <main className="flex-1 flex flex-col items-center justify-center text-center px-6 py-12 max-w-5xl mx-auto z-10">
        <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-white/5 border border-white/10 mb-8 hover:bg-white/10 transition-colors">
          <span className="w-2 h-2 rounded-full bg-primary animate-pulse"></span>
          <span className="text-xs font-semibold text-primary">Now integrated with Meta Cloud API</span>
        </div>

        <h1 className="text-4xl sm:text-6xl font-extrabold tracking-tight mb-6 leading-tight">
          Enterprise Bulk Messaging & <br />
          <span className="bg-gradient-to-r from-primary to-emerald-400 bg-clip-text text-transparent">
            WhatsApp Campaign Platform
          </span>
        </h1>

        <p className="text-muted-foreground text-lg sm:text-xl max-w-2xl mb-10 leading-relaxed">
          Scale your customer outreach, run automated sequence campaigns, manage real-time conversations, and get deep analytics — all powered by a robust enterprise engine.
        </p>

        <div className="flex flex-col sm:flex-row items-center gap-4 mb-16">
          <Link
            href={isAuthenticated ? "/dashboard" : "/register"}
            className="w-full sm:w-auto px-8 py-4 rounded-xl bg-gradient-to-tr from-primary to-emerald-500 text-primary-foreground font-semibold hover-scale shadow-lg shadow-primary/20 flex items-center justify-center gap-2"
          >
            Launch Free Trial <ArrowRight className="w-5 h-5" />
          </Link>
          <Link
            href="/login"
            className="w-full sm:w-auto px-8 py-4 rounded-xl bg-white/5 border border-white/10 font-semibold hover:bg-white/10 hover-scale flex items-center justify-center"
          >
            Schedule a Demo
          </Link>
        </div>

        {/* Feature Grid */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 w-full text-left">
          <div className="p-6 rounded-2xl glass-panel hover-scale">
            <div className="w-12 h-12 rounded-xl bg-primary/10 border border-primary/20 flex items-center justify-center mb-5">
              <Zap className="w-6 h-6 text-primary" />
            </div>
            <h3 className="text-lg font-bold mb-2">High-Volume Engine</h3>
            <p className="text-muted-foreground text-sm leading-relaxed">
              Send bulk messages to 100,000+ contacts concurrently. Batch processing with smart queue management and rate limiting ensures safe, compliant delivery.
            </p>
          </div>

          <div className="p-6 rounded-2xl glass-panel hover-scale">
            <div className="w-12 h-12 rounded-xl bg-primary/10 border border-primary/20 flex items-center justify-center mb-5">
              <BarChart3 className="w-6 h-6 text-primary" />
            </div>
            <h3 className="text-lg font-bold mb-2">Realtime Analytics</h3>
            <p className="text-muted-foreground text-sm leading-relaxed">
              Track sent, delivered, read, and response metrics in real-time. Gain actionable campaign insights, click rates, and daily message flows.
            </p>
          </div>

          <div className="p-6 rounded-2xl glass-panel hover-scale">
            <div className="w-12 h-12 rounded-xl bg-primary/10 border border-primary/20 flex items-center justify-center mb-5">
              <Shield className="w-6 h-6 text-primary" />
            </div>
            <h3 className="text-lg font-bold mb-2">Role-Based Security</h3>
            <p className="text-muted-foreground text-sm leading-relaxed">
              Organize campaigns with business administrators and team member hierarchies. Enterprise grade RBAC, AES-256 credentials encryption, and audit tracking.
            </p>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="px-6 py-8 border-t border-white/5 text-center text-xs text-muted-foreground z-10">
        <p>&copy; {new Date().getFullYear()} WhatsUp Enterprise. All rights reserved.</p>
      </footer>
    </div>
  );
}
