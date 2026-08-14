"use client";

import { ErrorFallback } from "@/components/error-fallback";

export default function AuthError({
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <ErrorFallback
      title="Sign-in page error"
      message="This screen failed to load. Retry, or go back to ChatBridge."
      onRetry={reset}
    />
  );
}
