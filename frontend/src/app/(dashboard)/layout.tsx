"use client";

import { useEffect, Suspense } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useAuthStore } from "@/store/authStore";
import DashboardShell from "@/components/layout/dashboard-shell";
import { BrandLogo } from "@/components/brand/logo";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const router = useRouter();
  const { isAuthenticated, isLoading, hasOrganization, initError } = useAuthStore();

  useEffect(() => {
    if (!isLoading && isAuthenticated && !hasOrganization) {
      router.push("/onboarding");
    }
  }, [isAuthenticated, isLoading, hasOrganization, router]);

  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background">
        <div className="h-10 w-10 animate-spin rounded-full border-4 border-primary/20 border-t-primary" />
      </div>
    );
  }

  if (!isAuthenticated || initError) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center bg-background px-6 text-center">
        <BrandLogo href="/" />
        <h1 className="mt-8 text-2xl font-semibold tracking-tight">Can&apos;t reach the API</h1>
        <p className="mt-2 max-w-md text-sm text-muted-foreground">
          {initError ||
            "This workspace needs a live API and a signed-in session. Marketing pages still work without it."}
        </p>
        <div className="mt-6 flex flex-wrap items-center justify-center gap-3">
          <Link
            href="/login"
            className="rounded-lg bg-[#14B8A6] px-4 py-2 text-sm font-medium text-white hover:opacity-90"
          >
            Sign in
          </Link>
          <Link
            href="/"
            className="rounded-lg border border-border px-4 py-2 text-sm font-medium hover:bg-accent"
          >
            Back to ChatBridge
          </Link>
        </div>
      </div>
    );
  }

  if (!hasOrganization) {
    return null;
  }

  return (
    <DashboardShell>
      <Suspense fallback={<div className="p-6 text-sm text-muted-foreground">Loading…</div>}>
        {children}
      </Suspense>
    </DashboardShell>
  );
}
