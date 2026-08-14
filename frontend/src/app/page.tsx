"use client";

import { useAuthStore } from "@/store/authStore";
import { MarketingNav } from "@/components/marketing/nav";
import { MarketingHero } from "@/components/marketing/hero";
import { MarketingFooter } from "@/components/marketing/footer";
import { LogoLoop } from "@/components/bits/logo-loop";
import { CampaignFrame } from "@/components/marketing/product-frame";
import { MarketingFeatures } from "@/components/marketing/features";
import { HowItWorks } from "@/components/marketing/how-it-works";
import { InboxShowcase } from "@/components/marketing/inbox-showcase";
import { AutomationSection } from "@/components/marketing/automations";
import { Testimonials } from "@/components/marketing/testimonials";
import { CtaBanner } from "@/components/marketing/cta-banner";

export default function Home() {
  const { isAuthenticated } = useAuthStore();

  return (
    <div className="min-h-screen bg-background">
      <MarketingNav authenticated={isAuthenticated} />
      <MarketingHero authenticated={isAuthenticated} />
      <LogoLoop />
      <MarketingFeatures />
      <HowItWorks />
      <InboxShowcase />
      <section id="campaigns" className="mx-auto grid max-w-6xl items-center gap-10 px-6 py-20 md:grid-cols-2">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.2em] text-primary">Campaigns</p>
          <h2 className="mt-2 text-3xl font-semibold tracking-tight">Wizard, then send</h2>
          <p className="mt-3 text-sm leading-relaxed text-muted-foreground">
            Audience, template or freeform, schedule, review. Track sent and delivered without a separate analytics product.
          </p>
        </div>
        <CampaignFrame />
      </section>
      <AutomationSection />
      <Testimonials />
      <CtaBanner authenticated={isAuthenticated} />
      <MarketingFooter />
    </div>
  );
}
