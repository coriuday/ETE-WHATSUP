import type { ReactNode } from "react";
import Link from "next/link";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Spotlight } from "@/components/bits/spotlight";
import { BrandLogo } from "@/components/brand/logo";

export function AuthSplit({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <div className="grid min-h-screen md:grid-cols-2">
      <div className="relative hidden overflow-hidden bg-[#042F2E] p-10 text-white md:flex md:flex-col">
        <Spotlight />
        <BrandLogo href="/" inverted />
        <div className="relative z-10 mt-auto max-w-sm">
          <p className="font-serif text-2xl font-semibold tracking-tight">Inbox, campaigns, mock provider.</p>
          <p className="mt-2 text-sm text-white/70">
            Sign in to assign conversations and launch bulk sends from the same ChatBridge workspace.
          </p>
        </div>
      </div>
      <div className="flex flex-col items-center justify-center p-6">
        <div className="mb-6 md:hidden">
          <BrandLogo href="/" />
        </div>
        <Card className="w-full max-w-md rounded-2xl shadow-sm">
          <CardHeader>
            <CardTitle>{title}</CardTitle>
            <CardDescription>{description}</CardDescription>
          </CardHeader>
          <CardContent>{children}</CardContent>
        </Card>
        <p className="mt-6 text-center text-xs text-muted-foreground">
          <Link href="/" className="hover:text-foreground">
            Back to ChatBridge
          </Link>
        </p>
      </div>
    </div>
  );
}
