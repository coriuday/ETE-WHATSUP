"use client";

import { useEffect, useState } from "react";
import { usePathname } from "next/navigation";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider } from "next-themes";
import { Toaster } from "react-hot-toast";
import { MuiProvider } from "@/components/mui/mui-provider";
import { useAuthStore } from "@/store/authStore";
import { Spinner } from "@/components/ui";

const AUTH_ONLY_BLOCK = [
  "/dashboard",
  "/activity",
  "/notifications",
  "/contacts",
  "/campaigns",
  "/inbox",
  "/templates",
  "/whatsapp",
  "/team",
  "/settings",
  "/schedules",
  "/automation",
  "/automations",
  "/billing",
  "/analytics",
  "/integrations",
  "/onboarding",
];

function shouldBlockOnAuth(pathname: string) {
  return AUTH_ONLY_BLOCK.some((p) => pathname === p || pathname.startsWith(`${p}/`));
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
    void initialize();
  }, [initialize]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      const state = useAuthStore.getState();
      if (!state.isLoading) return;
      useAuthStore.setState({
        isLoading: false,
        isAuthenticated: false,
        initError: "Can't reach the API. Sign in again when the service is available.",
      });
    }, 10000);
    return () => window.clearTimeout(timer);
  }, []);

  if (isLoading && shouldBlockOnAuth(pathname || "")) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center space-y-4 bg-background text-foreground">
        <Spinner className="h-12 w-12" />
        <p className="text-sm text-muted-foreground">Loading workspace…</p>
      </div>
    );
  }

  return (
    <ThemeProvider attribute="class" defaultTheme="light" enableSystem>
      <MuiProvider>
        <QueryClientProvider client={queryClient}>
          {children}
          <Toaster
            position="top-right"
            toastOptions={{
              duration: 4000,
              className: "!bg-card !text-foreground !border !border-border !rounded-lg !shadow-none",
            }}
          />
        </QueryClientProvider>
      </MuiProvider>
    </ThemeProvider>
  );
}
