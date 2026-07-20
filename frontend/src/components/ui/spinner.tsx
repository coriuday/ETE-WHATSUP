import { cn } from "@/lib/utils";

export function Spinner({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        "h-8 w-8 rounded-full border-4 border-primary/20 border-t-primary animate-spin",
        className
      )}
      role="status"
      aria-label="Loading"
    />
  );
}
