import Link from "next/link";
import { Star, ShieldCheck, FlaskConical } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Spotlight } from "@/components/bits/spotlight";
import { HeroProductMock } from "./product-frame";

export function MarketingHero({ authenticated }: { authenticated: boolean }) {
  return (
    <section className="relative overflow-hidden pb-12">
      <Spotlight />
      <div className="relative mx-auto grid max-w-6xl items-center gap-16 px-6 py-16 md:grid-cols-2 md:py-24 md:pb-28">
        <div>
          <p className="mb-4 inline-flex items-center gap-1.5 rounded-full border border-primary/20 bg-primary/10 px-3 py-1 text-xs font-medium text-primary">
            <Star className="h-3.5 w-3.5 fill-current" />
            ETE ChatBridge API · Inbox · Campaigns · Mock provider
          </p>
          <h1 className="text-4xl font-semibold tracking-tight text-foreground sm:text-5xl">
            Automate WhatsApp.{" "}
            <span className="text-primary">Grow your operations.</span>
          </h1>
          <p className="mt-4 max-w-md text-base leading-relaxed text-muted-foreground">
            Shared inbox, bulk campaigns, and automations — develop against a mock WhatsApp provider, then switch to Meta when you are ready.
          </p>
          <div className="mt-8 flex flex-wrap gap-3">
            <Button asChild size="lg" className="rounded-xl">
              <Link href={authenticated ? "/dashboard" : "/register"}>
                {authenticated ? "Open dashboard" : "Start Free Trial"}
              </Link>
            </Button>
            <Button asChild variant="outline" size="lg" className="rounded-xl">
              <Link href={authenticated ? "/inbox" : "/login"}>{authenticated ? "Open inbox" : "Log in"}</Link>
            </Button>
          </div>
          <div className="mt-6 flex flex-wrap gap-4 text-xs text-muted-foreground">
            <span className="inline-flex items-center gap-1.5">
              <ShieldCheck className="h-3.5 w-3.5 text-primary" />
              No credit card required
            </span>
            <span className="inline-flex items-center gap-1.5">
              <FlaskConical className="h-3.5 w-3.5 text-primary" />
              Mock provider included
            </span>
          </div>
        </div>
        <HeroProductMock />
      </div>
    </section>
  );
}
