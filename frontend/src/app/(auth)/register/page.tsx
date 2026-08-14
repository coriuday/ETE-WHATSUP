"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuthStore } from "@/store/authStore";
import toast from "react-hot-toast";
import { AuthSplit } from "@/components/auth/auth-split";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export default function Register() {
  const router = useRouter();
  const { setTokens, setUser } = useAuthStore();

  const [fullName, setFullName] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      const { api } = await import("@/lib/api");

      const nameParts = fullName.trim().split(" ");
      const firstName = nameParts[0] || "User";
      const lastName = nameParts.length > 1 ? nameParts.slice(1).join(" ") : "Name";

      await api.post("/auth/register", {
        first_name: firstName,
        last_name: lastName,
        email,
        password,
      });

      const loginRes = await api.post("/auth/login", { email, password });

      if (loginRes.data.data.requires_2fa) {
        toast.success("Account created! Please log in with 2FA.");
        router.push("/login");
        return;
      }

      const { access_token, refresh_token, user } = loginRes.data.data.tokens
        ? { ...loginRes.data.data.tokens, user: loginRes.data.data.user }
        : loginRes.data.data;

      setTokens(access_token, refresh_token);
      setUser(user);

      toast.success("Account created! Let's set up your organization.");
      router.push("/onboarding");
    } catch (err: unknown) {
      const axiosErr = err as { response?: { data?: { error?: { message?: string } | string } } };
      const errorMsg =
        (typeof axiosErr.response?.data?.error === "object"
          ? axiosErr.response?.data?.error?.message
          : axiosErr.response?.data?.error) || "Registration failed. Please try again.";
      toast.error(typeof errorMsg === "string" ? errorMsg : "Registration failed.");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <AuthSplit title="Create your account" description="Start with inbox and campaigns. Mock provider included.">
      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="fullName">Full name</Label>
          <Input id="fullName" required placeholder="Jane Doe" value={fullName} onChange={(e) => setFullName(e.target.value)} />
        </div>
        <div className="space-y-2">
          <Label htmlFor="email">Email</Label>
          <Input id="email" type="email" required placeholder="you@company.com" value={email} onChange={(e) => setEmail(e.target.value)} />
        </div>
        <div className="space-y-2">
          <Label htmlFor="password">Password</Label>
          <Input id="password" type="password" required minLength={8} value={password} onChange={(e) => setPassword(e.target.value)} />
          <p className="text-xs text-muted-foreground">Min 8 characters</p>
        </div>
        <Button type="submit" className="w-full" disabled={isLoading}>
          {isLoading ? "Please wait…" : "Create account"}
        </Button>
      </form>
      <p className="mt-4 text-center text-xs text-muted-foreground">
        Already have an account?{" "}
        <Link href="/login" className="font-medium text-primary hover:underline">
          Sign in
        </Link>
      </p>
    </AuthSplit>
  );
}
