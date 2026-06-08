"use client";

import { useEffect, useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "react-hot-toast";
import { useAuthStore } from "@/store/authStore";

export default function ClientProviders({
  children,
}: {
  children: React.ReactNode;
}) {
  const [queryClient] = useState(() => new QueryClient({
    defaultOptions: {
      queries: {
        refetchOnWindowFocus: false,
        retry: 1,
      },
    },
  }));

  const { initialize, isLoading } = useAuthStore();

  useEffect(() => {
    initialize();
  }, [initialize]);

  if (isLoading) {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen bg-slate-950 text-slate-100 space-y-4">
        <div className="relative w-16 h-16">
          <div className="absolute top-0 left-0 w-full h-full border-4 border-emerald-500/20 rounded-full"></div>
          <div className="absolute top-0 left-0 w-full h-full border-4 border-t-emerald-500 rounded-full animate-spin"></div>
        </div>
        <p className="text-slate-400 animate-pulse text-sm">Synchronizing dashboard...</p>
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
            borderRadius: "12px",
          },
        }}
      />
    </QueryClientProvider>
  );
}
