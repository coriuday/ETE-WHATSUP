"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useAuthStore } from "@/store/authStore";
import { cn } from "@/lib/utils";
import {
  MessageSquare,
  LayoutDashboard,
  Smartphone,
  Users,
  FileText,
  Send,
  Inbox,
  Calendar,
  Zap,
  Shield,
  CreditCard,
  Settings,
  Menu,
} from "lucide-react";

interface SidebarProps {
  isOpen: boolean;
  setIsOpen: (isOpen: boolean) => void;
}

export default function Sidebar({ isOpen, setIsOpen }: SidebarProps) {
  const pathname = usePathname();
  const { user } = useAuthStore();

  const isSuperAdmin = user?.role === "super_admin";
  const isBusinessAdmin = user?.role === "business_admin";

  const navigation = [
    { name: "Overview", href: "/dashboard", icon: LayoutDashboard },
    { name: "WhatsApp Accounts", href: "/whatsapp", icon: Smartphone },
    { name: "Contacts", href: "/contacts", icon: Users },
    { name: "Templates", href: "/templates", icon: FileText },
    { name: "Campaigns", href: "/campaigns", icon: Send },
    { name: "Inbox Chat", href: "/inbox", icon: Inbox },
    { name: "Schedules", href: "/schedules", icon: Calendar },
    { name: "Automations", href: "/automation", icon: Zap },
    { 
      name: "Team Management", 
      href: "/team", 
      icon: Shield,
      hidden: !isSuperAdmin && !isBusinessAdmin 
    },
    { 
      name: "Plans & Billing", 
      href: "/billing", 
      icon: CreditCard,
      hidden: !isSuperAdmin && !isBusinessAdmin 
    },
    { name: "Settings", href: "/settings", icon: Settings },
  ];

  return (
    <aside
      className={cn(
        "fixed inset-y-0 left-0 z-40 w-64 bg-slate-950 border-r border-white/5 flex flex-col transform transition-transform duration-300 md:translate-x-0 md:static md:h-screen",
        isOpen ? "translate-x-0" : "-translate-x-0 md:translate-x-0 hidden md:flex"
      )}
    >
      {/* Brand Header */}
      <div className="h-16 px-6 flex items-center justify-between border-b border-white/5">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-tr from-primary to-emerald-500 flex items-center justify-center shadow-lg shadow-primary/20">
            <MessageSquare className="w-4.5 h-4.5 text-primary-foreground" />
          </div>
          <span className="font-bold tracking-tight text-white">WhatsUp</span>
        </div>
        <button
          onClick={() => setIsOpen(false)}
          className="md:hidden text-muted-foreground hover:text-white"
        >
          <Menu className="w-5 h-5" />
        </button>
      </div>

      {/* Nav Menu */}
      <nav className="flex-1 px-4 py-6 overflow-y-auto space-y-1">
        {navigation.map((item) => {
          if (item.hidden) return null;
          const isActive = pathname.startsWith(item.href);
          const Icon = item.icon;

          return (
            <Link
              key={item.name}
              href={item.href}
              className={cn(
                "group flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm font-semibold transition-all duration-200",
                isActive
                  ? "bg-primary/10 text-primary border border-primary/10 shadow-sm"
                  : "text-muted-foreground hover:bg-white/5 hover:text-white"
              )}
            >
              <Icon className={cn("w-4.5 h-4.5 transition-colors", isActive ? "text-primary" : "text-muted-foreground/60 group-hover:text-white")} />
              <span>{item.name}</span>
            </Link>
          );
        })}
      </nav>

      {/* User Info Foot */}
      {user && (
        <div className="p-4 border-t border-white/5 bg-slate-950/50">
          <div className="flex items-center gap-3 px-2 py-1.5">
            <div className="w-9 h-9 rounded-full bg-gradient-to-tr from-primary to-accent-gradient flex items-center justify-center font-bold text-sm text-primary-foreground uppercase shadow-md shadow-primary/10">
              {(user.full_name || "").slice(0, 2)}
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-xs font-bold text-white truncate">{user.full_name}</p>
              <p className="text-[10px] text-muted-foreground capitalize mt-0.5 font-medium">
                {user.role.replace("_", " ")}
              </p>
            </div>
          </div>
        </div>
      )}
    </aside>
  );
}
