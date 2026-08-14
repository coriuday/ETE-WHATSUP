import type { Metadata } from "next";
import { Inter, Source_Serif_4 } from "next/font/google";
import ClientProviders from "./providers";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-sans",
});

const sourceSerif = Source_Serif_4({
  subsets: ["latin"],
  variable: "--font-serif",
});

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
    <html lang="en" className={`${inter.variable} ${sourceSerif.variable}`} suppressHydrationWarning>
      <body className="font-sans antialiased bg-background text-foreground min-h-screen">
        <ClientProviders>{children}</ClientProviders>
      </body>
    </html>
  );
}
