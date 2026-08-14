"use client";

import Link from "next/link";
import { BrandLogo } from "@/components/brand/logo";

export function ErrorFallback({
  title = "Something went wrong",
  message = "This page hit an unexpected error. You can retry or return home.",
  onRetry,
}: {
  title?: string;
  message?: string;
  onRetry?: () => void;
}) {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-background px-6 py-16 text-center">
      <BrandLogo href="/" />
      <h1 className="mt-8 text-2xl font-semibold tracking-tight text-foreground">{title}</h1>
      <p className="mt-2 max-w-md text-sm text-muted-foreground">{message}</p>
      <div className="mt-6 flex flex-wrap items-center justify-center gap-3">
        {onRetry ? (
          <button
            type="button"
            onClick={onRetry}
            className="rounded-lg bg-[#14B8A6] px-4 py-2 text-sm font-medium text-white hover:opacity-90"
          >
            Try again
          </button>
        ) : null}
        <Link
          href="/"
          className="rounded-lg border border-border px-4 py-2 text-sm font-medium text-foreground hover:bg-accent"
        >
          Back to ChatBridge
        </Link>
      </div>
    </div>
  );
}
