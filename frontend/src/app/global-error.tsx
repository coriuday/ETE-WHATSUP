"use client";

import { ErrorFallback } from "@/components/error-fallback";

export default function GlobalError({
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-background text-foreground antialiased">
        <ErrorFallback
          title="ChatBridge hit a problem"
          message="A last-resort error stopped this page. Retry, or return to the home page."
          onRetry={reset}
        />
      </body>
    </html>
  );
}
