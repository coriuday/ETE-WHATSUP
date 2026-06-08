"use client";

import { useState } from "react";
import { useAuthStore } from "@/store/authStore";
import { Menu, LogOut, Building, ChevronDown } from "lucide-react";
import { cn } from "@/lib/utils";

interface TopbarProps {
  onMenuClick: () => void;
}

export default function Topbar({ onMenuClick }: TopbarProps) {
  const { user, organization, logout } = useAuthStore();
  const [profileOpen, setProfileOpen] = useState(false);

  return (
    <header className="h-16 px-6 border-b border-white/5 bg-slate-950/20 backdrop-blur-md flex items-center justify-between sticky top-0 z-30">
      {/* Sidebar Toggle & Org Indicator */}
      <div className="flex items-center gap-4">
        <button
          onClick={onMenuClick}
          className="p-1.5 rounded-lg border border-white/10 text-muted-foreground hover:text-white hover:bg-white/5 md:hidden"
        >
          <Menu className="w-5 h-5" />
        </button>

        <div className="flex items-center gap-2 px-3 py-1.5 rounded-xl bg-white/5 border border-white/10 text-xs text-white">
          <Building className="w-4 h-4 text-primary" />
          <span className="font-semibold">{organization?.name || "No Organization"}</span>
        </div>
      </div>

      {/* User Actions */}
      <div className="flex items-center gap-4 relative">
        <button
          onClick={() => setProfileOpen(!profileOpen)}
          className="flex items-center gap-2 hover:opacity-90 transition-opacity focus:outline-none"
        >
          <div className="w-8 h-8 rounded-full bg-primary/10 border border-primary/20 flex items-center justify-center font-bold text-xs text-primary uppercase">
            {user?.fullName.slice(0, 2) || "U"}
          </div>
          <span className="hidden sm:block text-xs font-semibold text-white">{user?.fullName}</span>
          <ChevronDown className="w-4 h-4 text-muted-foreground" />
        </button>

        {/* Dropdown Menu */}
        {profileOpen && (
          <>
            <div
              className="fixed inset-0 z-40"
              onClick={() => setProfileOpen(false)}
            />
            <div className="absolute right-0 top-full mt-2 w-48 rounded-xl border border-white/10 bg-slate-900 p-2 shadow-2xl z-50 animate-in fade-in duration-100">
              <div className="px-3 py-2 border-b border-white/5 mb-1 text-left">
                <p className="text-xs font-bold text-white">{user?.fullName}</p>
                <p className="text-[10px] text-muted-foreground truncate">{user?.email}</p>
              </div>

              <button
                onClick={() => {
                  setProfileOpen(false);
                  logout();
                }}
                className="w-full flex items-center gap-2.5 px-3 py-2 text-xs font-semibold text-destructive hover:bg-destructive/10 rounded-lg transition-colors text-left"
              >
                <LogOut className="w-4 h-4" />
                Sign Out
              </button>
            </div>
          </>
        )}
      </div>
    </header>
  );
}
