import { describe, expect, it, vi } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { MarketingFooter } from "@/components/marketing/footer";
import { MarketingHero } from "@/components/marketing/hero";

vi.mock("next/link", () => ({
  default: ({ children, href }: { children: React.ReactNode; href: string }) => (
    <a href={href}>{children}</a>
  ),
}));

describe("marketing landing", () => {
  it("renders product copy instead of generic AI hero", () => {
    const html = renderToStaticMarkup(<MarketingHero authenticated={false} />);
    expect(html).toContain("Inbox");
    expect(html).toContain("Campaigns");
    expect(html).toContain("mock");
    expect(html).not.toContain("Launch Free Trial");
  });

  it("renders footer links", () => {
    const html = renderToStaticMarkup(<MarketingFooter />);
    expect(html).toContain("Docs");
    expect(html).toContain("ChatBridge");
  });
});
