"use client";

import * as React from "react";
import { useRouter } from "next/navigation";
import { Search } from "lucide-react";
import { Dialog } from "@radix-ui/react-dialog";
import * as DialogPrimitive from "@radix-ui/react-dialog";
import { cn } from "@/lib/utils";

const ROUTES = [
  { label: "Dashboard", href: "/dashboard" },
  { label: "Inbox", href: "/inbox" },
  { label: "Contacts", href: "/contacts" },
  { label: "Campaigns", href: "/campaigns" },
  { label: "Create campaign", href: "/campaigns/new" },
  { label: "Automations", href: "/automation" },
  { label: "Templates", href: "/templates" },
  { label: "WhatsApp", href: "/whatsapp" },
  { label: "Analytics", href: "/analytics" },
  { label: "Settings", href: "/settings" },
];

export function CommandMenu({ open, onOpenChange }: { open: boolean; onOpenChange: (v: boolean) => void }) {
  const router = useRouter();
  const [q, setQ] = React.useState("");
  const items = ROUTES.filter((r) => r.label.toLowerCase().includes(q.toLowerCase()));

  React.useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        onOpenChange(!open);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onOpenChange]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogPrimitive.Portal>
        <DialogPrimitive.Overlay className="fixed inset-0 z-50 bg-foreground/20" />
        <DialogPrimitive.Content className="fixed left-1/2 top-[20%] z-50 w-full max-w-lg -translate-x-1/2 rounded-xl border border-border bg-card p-2">
          <DialogPrimitive.Title className="sr-only">Jump to</DialogPrimitive.Title>
          <div className="flex items-center gap-2 border-b border-border px-3 py-2">
            <Search className="h-4 w-4 text-muted-foreground" />
            <input
              autoFocus
              value={q}
              onChange={(e) => setQ(e.target.value)}
              placeholder="Jump to a page…"
              className="h-8 w-full bg-transparent text-sm outline-none"
            />
          </div>
          <ul className="max-h-72 overflow-y-auto p-1">
            {items.map((item) => (
              <li key={item.href}>
                <button
                  className={cn(
                    "flex w-full rounded-md px-3 py-2 text-left text-sm hover:bg-accent"
                  )}
                  onClick={() => {
                    onOpenChange(false);
                    router.push(item.href);
                  }}
                >
                  {item.label}
                </button>
              </li>
            ))}
            {!items.length && <li className="px-3 py-6 text-center text-sm text-muted-foreground">No matches</li>}
          </ul>
        </DialogPrimitive.Content>
      </DialogPrimitive.Portal>
    </Dialog>
  );
}
