import type { Metadata } from "next";
import ClientProviders from "./providers";
import "./globals.css";

export const metadata: Metadata = {
  title: "ChatBridge — WhatsApp inbox, campaigns, and automations",
  description:
    "ETE ChatBridge API: shared inbox, bulk campaigns, contacts, and automations. Develop against a mock WhatsApp provider, then switch to Meta.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className="font-sans antialiased bg-background text-foreground min-h-screen">
        <ClientProviders>{children}</ClientProviders>
      </body>
    </html>
  );
}
