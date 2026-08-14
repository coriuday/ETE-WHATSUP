import Link from "next/link";
import { BrandLogo, BrandLogoImage } from "@/components/brand/logo";

const COLUMNS = [
  {
    title: "Product",
    links: [
      { href: "#inbox", label: "Inbox" },
      { href: "#campaigns", label: "Campaigns" },
      { href: "#automations", label: "Automations" },
      { href: "#features", label: "Contacts & analytics" },
    ],
  },
  {
    title: "Resources",
    links: [
      { href: "/login", label: "Docs" },
      { href: "#how-it-works", label: "How it works" },
      { href: "/register", label: "Start trial" },
    ],
  },
  {
    title: "Company",
    links: [
      { href: "/", label: "ChatBridge" },
      { href: "/login", label: "Log in" },
      { href: "/register", label: "Create account" },
    ],
  },
];

export function MarketingFooter() {
  return (
    <footer className="bg-[#042F2E] text-white">
      <div className="mx-auto grid max-w-6xl gap-10 px-6 py-14 md:grid-cols-[1.4fr_1fr_1fr_1fr]">
        <div>
          <BrandLogo href="/" inverted />
          <p className="mt-4 max-w-xs text-sm text-white/65">
            ETE ChatBridge API — shared inbox, campaigns, and automations for WhatsApp, with a mock provider for development.
          </p>
          <BrandLogoImage className="mt-6 h-8 opacity-90" />
        </div>
        {COLUMNS.map((col) => (
          <div key={col.title}>
            <p className="text-sm font-semibold">{col.title}</p>
            <ul className="mt-3 space-y-2 text-sm text-white/65">
              {col.links.map((link) => (
                <li key={link.label}>
                  {link.href.startsWith("/") ? (
                    <Link href={link.href} className="hover:text-white">
                      {link.label}
                    </Link>
                  ) : (
                    <a href={link.href} className="hover:text-white">
                      {link.label}
                    </a>
                  )}
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <div className="border-t border-white/10 px-6 py-4 text-center text-xs text-white/50">
        © {new Date().getFullYear()} ChatBridge · WhatsUp workspace
      </div>
    </footer>
  );
}
