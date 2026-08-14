"use client";

import { useState } from "react";
import Link from "next/link";
import toast from "react-hot-toast";
import { AuthSplit } from "@/components/auth/auth-split";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export default function ForgotPassword() {
  const [email, setEmail] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isSent, setIsSent] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      const { api } = await import("@/lib/api");
      await api.post("/auth/forgot-password", { email });
      setIsSent(true);
      toast.success("Reset link sent successfully!");
    } catch (err: unknown) {
      const axiosErr = err as { response?: { data?: { error?: string } } };
      toast.error(axiosErr.response?.data?.error || "Failed to send reset link. Please verify your email.");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <AuthSplit title="Reset password" description="We’ll email a reset link if the account exists.">
      {!isSent ? (
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <Input
              id="email"
              type="email"
              required
              placeholder="you@company.com"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>
          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? "Sending…" : "Send reset link"}
          </Button>
        </form>
      ) : (
        <p className="text-sm text-muted-foreground">
          If the email is registered, we sent reset instructions to {email}.
        </p>
      )}
      <p className="mt-4 text-center text-xs">
        <Link href="/login" className="font-medium text-primary hover:underline">
          Back to sign in
        </Link>
      </p>
    </AuthSplit>
  );
}
