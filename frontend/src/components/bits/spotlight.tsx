"use client";

import { cn } from "@/lib/utils";

/** Restrained spotlight — React Bits–style marketing effect, landing/auth only. */
export function Spotlight({ className }: { className?: string }) {
  return (
    <div
      aria-hidden
      className={cn("pointer-events-none absolute inset-0 overflow-hidden", className)}
    >
      <div className="absolute -top-24 left-1/4 h-[420px] w-[420px] rounded-full bg-primary/[0.07] blur-3xl" />
      <div className="absolute bottom-0 right-0 h-[280px] w-[280px] rounded-full bg-primary/[0.04] blur-3xl" />
    </div>
  );
}
