"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useAuthStore } from "@/store/authStore";
import { cn } from "@/lib/utils";
import {
  LayoutDashboard,
  Inbox,
  Users,
  Send,
  Zap,
  FileText,
  Calendar,
  Smartphone,
  Shield,
  BarChart3,
  Plug,
  CreditCard,
  Settings,
  Menu,
  PanelLeft,
} from "lucide-react";
import { BrandLogo, BrandMark } from "@/components/brand/logo";

interface SidebarProps {
  isOpen: boolean;
  setIsOpen: (isOpen: boolean) => void;
  collapsed: boolean;
  onToggleCollapse: () => void;
}

type NavItem = {
  name: string;
  href: string;
  soon?: boolean;
  admin?: boolean;
};

type NavGroup = {
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  items: NavItem[];
};

export default function Sidebar({ isOpen, setIsOpen, collapsed, onToggleCollapse }: SidebarProps) {
  const pathname = usePathname();
  const { user } = useAuthStore();
  const isAdmin = user?.role === "super_admin" || user?.role === "business_admin";

  const groups: NavGroup[] = [
    {
      label: "Overview",
      icon: LayoutDashboard,
      items: [
        { name: "Dashboard", href: "/dashboard" },
        { name: "Activity", href: "/activity", soon: true },
        { name: "Notifications", href: "/notifications", soon: true },
      ],
    },
    {
      label: "Inbox",
      icon: Inbox,
      items: [
        { name: "All", href: "/inbox" },
        { name: "Unassigned", href: "/inbox/unassigned" },
        { name: "Mine", href: "/inbox/mine" },
        { name: "Closed", href: "/inbox/closed" },
      ],
    },
    {
      label: "Contacts",
      icon: Users,
      items: [
        { name: "All Contacts", href: "/contacts" },
        { name: "Lists", href: "/contacts?tab=lists" },
        { name: "Tags", href: "/contacts?tab=tags" },
        { name: "Segments", href: "/contacts?tab=segments" },
        { name: "Import", href: "/contacts?tab=import" },
        { name: "Export", href: "/contacts?tab=export" },
      ],
    },
    {
      label: "Campaigns",
      icon: Send,
      items: [
        { name: "All Campaigns", href: "/campaigns" },
        { name: "Create Campaign", href: "/campaigns/new" },
        { name: "Drafts", href: "/campaigns?status=draft" },
        { name: "Scheduled", href: "/campaigns?status=scheduled" },
        { name: "Running", href: "/campaigns?status=running" },
        { name: "Completed", href: "/campaigns?status=completed" },
      ],
    },
    {
      label: "Automations",
      icon: Zap,
      items: [{ name: "Flows", href: "/automation" }, { name: "Execution Logs", href: "/automation?tab=logs" }],
    },
    {
      label: "Templates",
      icon: FileText,
      items: [
        { name: "WhatsApp Templates", href: "/templates" },
        { name: "Quick Replies", href: "/templates?tab=quick-replies" },
      ],
    },
    {
      label: "Schedules",
      icon: Calendar,
      items: [{ name: "Campaign Schedules", href: "/schedules" }],
    },
    {
      label: "WhatsApp",
      icon: Smartphone,
      items: [
        { name: "Accounts", href: "/whatsapp" },
        { name: "Health", href: "/whatsapp?tab=health" },
      ],
    },
    {
      label: "Team",
      icon: Shield,
      items: [
        { name: "Members", href: "/team", admin: true },
        { name: "Audit Log", href: "/team?tab=audit", admin: true },
      ],
    },
    {
      label: "Analytics",
      icon: BarChart3,
      items: [
        { name: "Overview", href: "/analytics" },
        { name: "Campaign Analytics", href: "/analytics?tab=campaigns" },
        { name: "Delivery Analytics", href: "/analytics?tab=delivery" },
      ],
    },
    {
      label: "Integrations",
      icon: Plug,
      items: [
        { name: "n8n", href: "/integrations" },
        { name: "Webhooks", href: "/integrations?tab=webhooks" },
      ],
    },
    {
      label: "Billing",
      icon: CreditCard,
      items: [
        { name: "Plan & Usage", href: "/billing", admin: true },
        { name: "Invoices", href: "/billing?tab=invoices", admin: true, soon: true },
      ],
    },
    {
      label: "Settings",
      icon: Settings,
      items: [
        { name: "Workspace", href: "/settings" },
        { name: "Security", href: "/settings?tab=security" },
      ],
    },
  ];

  return (
    <aside
      className={cn(
        "fixed inset-y-0 left-0 z-40 flex flex-col border-r border-border bg-card md:static md:h-screen",
        collapsed ? "md:w-16" : "w-60",
        isOpen ? "translate-x-0 w-60" : "-translate-x-full md:translate-x-0",
        "transition-[width,transform]"
      )}
    >
      <div className="flex h-14 items-center justify-between border-b border-border px-3">
        <Link href="/dashboard" className="flex items-center gap-2 overflow-hidden">
          {collapsed ? (
            <BrandMark />
          ) : (
            <BrandLogo href={null} wordmark="WhatsUp" />
          )}
        </Link>
        <button
          onClick={() => setIsOpen(false)}
          className="text-muted-foreground md:hidden"
          aria-label="Close navigation"
        >
          <Menu className="h-5 w-5" />
        </button>
        <button
          type="button"
          onClick={onToggleCollapse}
          className="hidden rounded-md p-1 text-muted-foreground hover:bg-accent md:inline-flex"
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          <PanelLeft className="h-4 w-4" />
        </button>
      </div>

      <nav className="flex-1 overflow-y-auto px-2 py-4">
        {groups.map((group) => {
          const visible = group.items.filter((i) => !i.admin || isAdmin);
          if (!visible.length) return null;
          const Icon = group.icon;
          const primaryHref = visible[0].href;
          const groupActive = visible.some((item) => pathname === item.href.split("?")[0]);
          if (collapsed) {
            return (
              <Link
                key={group.label}
                href={primaryHref}
                title={group.label}
                className={cn(
                  "mb-1 flex h-9 items-center justify-center rounded-lg",
                  groupActive ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-accent"
                )}
              >
                <Icon className="h-4 w-4" />
              </Link>
            );
          }
          return (
            <div key={group.label} className="mb-5">
              <p className="mb-1.5 px-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
                {group.label}
              </p>
              <ul className="space-y-0.5">
                {visible.map((item) => {
                  const path = item.href.split("?")[0];
                  const active = !item.soon && pathname === path;
                  return (
                    <li key={item.name}>
                      {item.soon ? (
                        <span className="flex items-center justify-between rounded-lg px-2 py-1.5 text-sm text-muted-foreground/70">
                          {item.name}
                          <span className="rounded bg-muted px-1.5 py-0.5 text-[10px] font-medium">Soon</span>
                        </span>
                      ) : (
                        <Link
                          href={item.href}
                          onClick={() => setIsOpen(false)}
                          className={cn(
                            "block rounded-lg px-2 py-1.5 text-sm",
                            active ? "bg-primary/10 font-medium text-primary" : "text-foreground hover:bg-accent"
                          )}
                        >
                          {item.name}
                        </Link>
                      )}
                    </li>
                  );
                })}
              </ul>
            </div>
          );
        })}
      </nav>
    </aside>
  );
}
