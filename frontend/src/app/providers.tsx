"use client";

import { useEffect, useState } from "react";
import { usePathname } from "next/navigation";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "react-hot-toast";
import { useAuthStore } from "@/store/authStore";
import { Spinner } from "@/components/ui";

const AUTH_ONLY_BLOCK = ["/dashboard", "/contacts", "/campaigns", "/inbox", "/templates", "/whatsapp", "/team", "/settings", "/schedules", "/automation", "/billing", "/onboarding"];

function shouldBlockOnAuth(pathname: string) {
  return AUTH_ONLY_BLOCK.some(
    (p) => pathname === p || pathname.startsWith(`${p}/`)
  );
}

export default function ClientProviders({
  children,
}: {
  children: React.ReactNode;
}) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            refetchOnWindowFocus: false,
            retry: 1,
          },
        },
      })
  );

  const { initialize, isLoading } = useAuthStore();
  const pathname = usePathname();

  useEffect(() => {
    initialize();
  }, [initialize]);

  if (isLoading && shouldBlockOnAuth(pathname || "")) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center space-y-4 bg-background text-foreground">
        <Spinner className="h-12 w-12" />
        <p className="animate-pulse text-sm text-muted-foreground">
          Loading workspace…
        </p>
      </div>
    );
  }

  return (
    <QueryClientProvider client={queryClient}>
      {children}
      <Toaster
        position="top-right"
        toastOptions={{
          duration: 4000,
          style: {
            background: "#0d1423",
            color: "#f1f5f9",
            border: "1px solid rgba(255,255,255,0.08)",
            borderRadius: "8px",
          },
        }}
      />
    </QueryClientProvider>
  );
}
