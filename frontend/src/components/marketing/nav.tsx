"use client";

import Link from "next/link";
import { BrandLogo } from "@/components/brand/logo";
import { Button } from "@/components/ui/button";

const LINKS = [
  { href: "#features", label: "Features" },
  { href: "#solutions", label: "Solutions" },
  { href: "#how-it-works", label: "How it works" },
  { href: "#inbox", label: "Inbox" },
];

export function MarketingNav({ authenticated }: { authenticated: boolean }) {
  return (
    <header className="sticky top-0 z-50 border-b border-border/80 bg-white/90 backdrop-blur-md dark:bg-background/90">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-6">
        <BrandLogo />
        <nav className="hidden items-center gap-7 text-sm text-muted-foreground md:flex">
          {LINKS.map((link) => (
            <a key={link.href} href={link.href} className="hover:text-foreground">
              {link.label}
            </a>
          ))}
        </nav>
        <div className="flex items-center gap-2">
          {authenticated ? (
            <Button asChild size="sm">
              <Link href="/dashboard">Open app</Link>
            </Button>
          ) : (
            <>
              <Button asChild variant="ghost" size="sm">
                <Link href="/login">Log in</Link>
              </Button>
              <Button asChild size="sm" className="rounded-xl px-4">
                <Link href="/register">Start Free Trial</Link>
              </Button>
            </>
          )}
        </div>
      </div>
    </header>
  );
}
