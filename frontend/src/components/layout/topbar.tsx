"use client";

import { useState } from "react";
import { useAuthStore } from "@/store/authStore";
import { Menu, LogOut, Building, Moon, Sun, Search } from "lucide-react";
import { useTheme } from "next-themes";
import { Avatar, Dropdown, DropdownContent, DropdownItem, DropdownTrigger, IconButton, CommandMenu } from "@/components/ui";

interface TopbarProps {
  onMenuClick: () => void;
}

export default function Topbar({ onMenuClick }: TopbarProps) {
  const { user, organization, logout } = useAuthStore();
  const { theme, setTheme } = useTheme();
  const [commandOpen, setCommandOpen] = useState(false);

  return (
    <header className="sticky top-0 z-30 flex h-14 items-center justify-between border-b border-border bg-card px-4">
      <div className="flex items-center gap-3">
        <button
          onClick={onMenuClick}
          className="rounded-lg border border-border p-1.5 text-muted-foreground md:hidden"
          aria-label="Open navigation"
        >
          <Menu className="h-5 w-5" />
        </button>
        <div className="flex items-center gap-2 rounded-lg border border-border px-2.5 py-1 text-xs">
          <Building className="h-3.5 w-3.5 text-primary" />
          <span className="font-medium">{organization?.name || "No workspace"}</span>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={() => setCommandOpen(true)}
          className="hidden h-9 items-center gap-2 rounded-lg border border-border px-3 text-sm text-muted-foreground hover:bg-accent sm:flex"
        >
          <Search className="h-4 w-4" />
          Jump to…
          <kbd className="rounded border border-border bg-muted px-1.5 text-[10px]">⌘K</kbd>
        </button>
        <IconButton aria-label="Open command" className="sm:hidden" onClick={() => setCommandOpen(true)}>
          <Search className="h-4 w-4" />
        </IconButton>
        <IconButton aria-label="Toggle theme" onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>
          <Sun className="h-4 w-4 dark:hidden" />
          <Moon className="hidden h-4 w-4 dark:block" />
        </IconButton>
        <Dropdown>
          <DropdownTrigger asChild>
            <button className="flex items-center gap-2 rounded-lg px-1.5 py-1 hover:bg-accent">
              <Avatar name={user?.fullName} />
              <span className="hidden text-xs font-medium sm:block">{user?.fullName}</span>
            </button>
          </DropdownTrigger>
          <DropdownContent align="end">
            <DropdownItem onSelect={() => logout()}>
              <LogOut className="h-4 w-4" />
              Sign out
            </DropdownItem>
          </DropdownContent>
        </Dropdown>
      </div>
      <CommandMenu open={commandOpen} onOpenChange={setCommandOpen} />
    </header>
  );
}
