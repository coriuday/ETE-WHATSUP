"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuthStore } from "@/store/authStore";
import { Eye, EyeOff, ShieldCheck } from "lucide-react";
import toast from "react-hot-toast";
import { getErrorMessage } from "@/lib/api";
import { AuthSplit } from "@/components/auth/auth-split";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export default function Login() {
  const router = useRouter();
  const { setTokens, setUser, setOrganization } = useAuthStore();

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [requires2Fa, setRequires2Fa] = useState(false);
  const [twoFactorToken, setTwoFactorToken] = useState("");
  const [code, setCode] = useState("");

  const handleLoginSuccess = async (accessToken: string, refreshToken: string, user: unknown) => {
    setTokens(accessToken, refreshToken);
    setUser(user as Parameters<typeof setUser>[0]);

    try {
      const { api } = await import("@/lib/api");
      const orgRes = await api.get("/organizations");
      const orgs = orgRes.data.data.organizations || [];
      if (orgs.length > 0) {
        setOrganization(orgs[0]);
        toast.success("Welcome back!");
        router.push("/dashboard");
      } else {
        toast.success("Welcome! Let's set up your organization.");
        router.push("/onboarding");
      }
    } catch {
      toast.success("Welcome back!");
      router.push("/dashboard");
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      const { api } = await import("@/lib/api");

      if (requires2Fa) {
        const res = await api.post("/auth/2fa/verify", {
          token: twoFactorToken,
          code,
        });

        const { accessToken, refreshToken, user } = res.data.data;
        await handleLoginSuccess(accessToken, refreshToken, user);
      } else {
        const res = await api.post("/auth/login", { email, password });

        if (res.data.data.requires_2fa) {
          setRequires2Fa(true);
          setTwoFactorToken(res.data.data.token);
          toast.success("Please enter your 2FA verification code");
        } else {
          const { access_token, refresh_token, user } = res.data.data.tokens
            ? { ...res.data.data.tokens, user: res.data.data.user }
            : res.data.data;
          await handleLoginSuccess(access_token, refresh_token, user);
        }
      }
    } catch (err: unknown) {
      toast.error(getErrorMessage(err, "Invalid credentials. Please try again."));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <AuthSplit title="Sign in" description="Manage inbox, campaigns, and your mock provider.">
      <form onSubmit={handleSubmit} className="space-y-4">
        {!requires2Fa ? (
          <>
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
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="password">Password</Label>
                <Link href="/forgot-password" className="text-xs font-medium text-primary hover:underline">
                  Forgot password?
                </Link>
              </div>
              <div className="relative">
                <Input
                  id="password"
                  type={showPassword ? "text" : "password"}
                  required
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground"
                  aria-label="Toggle password visibility"
                >
                  {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
            </div>
          </>
        ) : (
          <div className="space-y-2">
            <div className="flex items-start gap-2 rounded-lg border border-border bg-muted/50 p-3 text-xs text-muted-foreground">
              <ShieldCheck className="mt-0.5 h-4 w-4 text-primary" />
              Two-factor authentication is enabled. Enter the code from your authenticator app.
            </div>
            <Label htmlFor="code">Authenticator code</Label>
            <Input
              id="code"
              required
              placeholder="123456"
              maxLength={6}
              value={code}
              onChange={(e) => setCode(e.target.value)}
              className="text-center tracking-widest"
            />
          </div>
        )}
        <Button type="submit" className="w-full" disabled={isLoading}>
          {isLoading ? "Please wait…" : requires2Fa ? "Verify code" : "Sign in"}
        </Button>
      </form>
      {!requires2Fa && (
        <p className="mt-4 text-center text-xs text-muted-foreground">
          Don&apos;t have an account?{" "}
          <Link href="/register" className="font-medium text-primary hover:underline">
            Create an account
          </Link>
        </p>
      )}
    </AuthSplit>
  );
}
