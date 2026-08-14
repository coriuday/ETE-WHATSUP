"use client";

import { Suspense, useEffect, useState } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { CheckCircle, XCircle, Loader2 } from "lucide-react";
import { AuthSplit } from "@/components/auth/auth-split";
import { Button } from "@/components/ui/button";

export default function VerifyEmail() {
  return (
    <Suspense fallback={null}>
      <VerifyEmailInner />
    </Suspense>
  );
}

function VerifyEmailInner() {
  const searchParams = useSearchParams();
  const token = searchParams.get("token");

  const [status, setStatus] = useState<"loading" | "success" | "error">("loading");
  const [message, setMessage] = useState("Verifying your email address...");

  useEffect(() => {
    if (!token) {
      setStatus("error");
      setMessage("Verification token is missing.");
      return;
    }

    const verify = async () => {
      try {
        const { api } = await import("@/lib/api");
        await api.get(`/auth/verify-email?token=${token}`);
        setStatus("success");
        setMessage("Your email has been verified successfully! You can now log in.");
      } catch (err: unknown) {
        const axiosErr = err as { response?: { data?: { error?: string } } };
        setStatus("error");
        setMessage(axiosErr.response?.data?.error || "Email verification failed or the token has expired.");
      }
    };

    verify();
  }, [token]);

  return (
    <AuthSplit title="Email verification" description={message}>
      <div className="flex flex-col items-center gap-4 py-2 text-center">
        {status === "loading" && <Loader2 className="h-8 w-8 animate-spin text-primary" />}
        {status === "success" && <CheckCircle className="h-8 w-8 text-primary" />}
        {status === "error" && <XCircle className="h-8 w-8 text-destructive" />}
        <Button asChild className="w-full">
          <Link href="/login">Go to sign in</Link>
        </Button>
      </div>
    </AuthSplit>
  );
}
