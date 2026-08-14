"use client";

import { ShoppingBag, Building2, GraduationCap, HeartPulse, Headset, Store } from "lucide-react";

const INDUSTRIES = [
  { name: "E-commerce", icon: ShoppingBag },
  { name: "Real estate", icon: Building2 },
  { name: "Education", icon: GraduationCap },
  { name: "Healthcare", icon: HeartPulse },
  { name: "Support", icon: Headset },
  { name: "Retail", icon: Store },
];

export function LogoLoop() {
  const items = [...INDUSTRIES, ...INDUSTRIES];
  return (
    <div id="solutions" className="overflow-hidden border-y border-border py-8">
      <p className="mb-5 text-center text-xs font-medium uppercase tracking-wider text-muted-foreground">
        Built for teams across industries
      </p>
      <div className="flex w-max animate-marquee gap-12 px-6">
        {items.map((item, i) => {
          const Icon = item.icon;
          return (
            <span
              key={`${item.name}-${i}`}
              className="inline-flex items-center gap-2 text-sm font-medium text-muted-foreground"
            >
              <Icon className="h-4 w-4 text-primary" />
              {item.name}
            </span>
          );
        })}
      </div>
    </div>
  );
}
