"use client";

import { useEffect, Suspense } from "react";
import { useRouter } from "next/navigation";
import { useAuthStore } from "@/store/authStore";
import DashboardShell from "@/components/layout/dashboard-shell";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const router = useRouter();
  const { isAuthenticated, isLoading, hasOrganization } = useAuthStore();

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.push("/login");
    } else if (!isLoading && isAuthenticated && !hasOrganization) {
      router.push("/onboarding");
    }
  }, [isAuthenticated, isLoading, hasOrganization, router]);

  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background">
        <div className="w-10 h-10 border-4 border-primary/20 border-t-primary rounded-full animate-spin" />
      </div>
    );
  }

  if (!isAuthenticated || !hasOrganization) {
    return null; // will redirect via useEffect
  }

  return (
    <DashboardShell>
      <Suspense fallback={<div className="p-6 text-sm text-muted-foreground">Loading…</div>}>
        {children}
      </Suspense>
    </DashboardShell>
  );
}
