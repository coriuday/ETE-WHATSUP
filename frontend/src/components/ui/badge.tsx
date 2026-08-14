import type { HTMLAttributes, ReactNode } from "react";
import { cn } from "@/lib/utils";

export function Badge({
  className,
  variant = "default",
  ...props
}: HTMLAttributes<HTMLSpanElement> & {
  variant?: "default" | "secondary" | "outline" | "success" | "warning" | "danger";
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-medium",
        variant === "default" && "border-transparent bg-primary/10 text-primary",
        variant === "secondary" && "border-transparent bg-muted text-muted-foreground",
        variant === "outline" && "border-border text-foreground",
        variant === "success" && "border-transparent bg-success/10 text-success",
        variant === "warning" && "border-transparent bg-warning/10 text-warning",
        variant === "danger" && "border-transparent bg-destructive/10 text-destructive",
        className
      )}
      {...props}
    />
  );
}

export function StatusBadge({ status }: { status: string }) {
  const map: Record<string, "success" | "warning" | "danger" | "secondary" | "default"> = {
    sent: "default",
    delivered: "success",
    read: "success",
    failed: "danger",
    queued: "secondary",
    processing: "warning",
    retrying: "warning",
    cancelled: "secondary",
    running: "default",
    completed: "success",
    scheduled: "warning",
    draft: "secondary",
    open: "default",
    resolved: "success",
    connected: "success",
    disconnected: "secondary",
    mock: "warning",
    active: "success",
  };
  return <Badge variant={map[status] || "secondary"}>{status}</Badge>;
}

export function Tag({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-md bg-muted px-2 py-0.5 text-xs text-muted-foreground",
        className
      )}
    >
      {children}
    </span>
  );
}
