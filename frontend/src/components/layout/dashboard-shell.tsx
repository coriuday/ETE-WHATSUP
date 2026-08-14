"use client";

import { useState } from "react";
import { usePathname } from "next/navigation";
import Sidebar from "./sidebar";
import Topbar from "./topbar";
import { cn } from "@/lib/utils";

interface DashboardShellProps {
  children: React.ReactNode;
}

export default function DashboardShell({ children }: DashboardShellProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [collapsed, setCollapsed] = useState(false);
  const pathname = usePathname();
  const fullBleed = pathname.startsWith("/inbox");

  return (
    <div className="flex h-screen overflow-hidden bg-background">
      <Sidebar
        isOpen={sidebarOpen}
        setIsOpen={setSidebarOpen}
        collapsed={collapsed}
        onToggleCollapse={() => setCollapsed((v) => !v)}
      />
      <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
        <Topbar onMenuClick={() => setSidebarOpen(true)} />
        <main
          className={cn(
            "relative flex-1",
            fullBleed ? "overflow-hidden" : "overflow-y-auto px-6 py-6"
          )}
        >
          <div className={cn(fullBleed ? "h-full" : "relative z-10 mx-auto max-w-7xl space-y-6")}>
            {children}
          </div>
        </main>
      </div>
    </div>
  );
}
