import type { Metadata } from "next";
import { Outfit } from "next/font/google";
import ClientProviders from "./providers";
import "./globals.css";

const outfit = Outfit({
  subsets: ["latin"],
  variable: "--font-sans",
});

export const metadata: Metadata = {
  title: "WhatsUp — Enterprise WhatsApp Bulk Messaging Platform",
  description: "Scale your customer outreach, run automated sequence campaigns, and manage real-time conversations.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className={`${outfit.variable} dark`} suppressHydrationWarning>
      <body className="font-sans antialiased bg-background text-foreground min-h-screen" suppressHydrationWarning>
        <ClientProviders>
          {children}
        </ClientProviders>
      </body>
    </html>
  );
}

