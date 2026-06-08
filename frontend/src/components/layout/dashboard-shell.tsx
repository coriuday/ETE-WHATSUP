"use client";

import { useState } from "react";
import Sidebar from "./sidebar";
import Topbar from "./topbar";

interface DashboardShellProps {
  children: React.ReactNode;
}

export default function DashboardShell({ children }: DashboardShellProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="flex h-screen overflow-hidden bg-slate-950">
      {/* Sidebar Navigation */}
      <Sidebar isOpen={sidebarOpen} setIsOpen={setSidebarOpen} />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {/* Top Header */}
        <Topbar onMenuClick={() => setSidebarOpen(true)} />

        {/* Scrollable page body */}
        <main className="flex-1 overflow-y-auto px-6 py-8 relative">
          {/* Background overlay grids */}
          <div className="absolute top-0 right-0 w-[300px] h-[300px] bg-primary/5 blur-[80px] pointer-events-none rounded-full"></div>
          
          <div className="max-w-7xl mx-auto space-y-6 relative z-10">
            {children}
          </div>
        </main>
      </div>
    </div>
  );
}
