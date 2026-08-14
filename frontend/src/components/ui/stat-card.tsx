import { cn } from "@/lib/utils";

export function Avatar({
  name,
  className,
}: {
  name?: string;
  className?: string;
}) {
  const initials = (name || "U")
    .split(" ")
    .map((p) => p[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();
  return (
    <div
      className={cn(
        "flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-xs font-semibold text-primary",
        className
      )}
    >
      {initials}
    </div>
  );
}

export function AvatarGroup({ names }: { names: string[] }) {
  return (
    <div className="flex -space-x-2">
      {names.slice(0, 4).map((n) => (
        <Avatar key={n} name={n} className="border-2 border-background" />
      ))}
    </div>
  );
}

export function StatCard({
  label,
  value,
  hint,
}: {
  label: string;
  value: React.ReactNode;
  hint?: string;
}) {
  return (
    <div className="rounded-xl border border-border bg-card p-4">
      <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">{label}</p>
      <p className="mt-2 text-2xl font-semibold tracking-tight">{value}</p>
      {hint ? <p className="mt-1 text-xs text-muted-foreground">{hint}</p> : null}
    </div>
  );
}

export function ChartCard({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="rounded-xl border border-border bg-card p-4">
      <h3 className="mb-4 text-sm font-semibold">{title}</h3>
      {children}
    </div>
  );
}

export function ComingSoon({ title, description }: { title: string; description?: string }) {
  return (
    <div className="rounded-xl border border-dashed border-border bg-card p-10 text-center">
      <h2 className="text-lg font-semibold">{title}</h2>
      <p className="mt-2 text-sm text-muted-foreground">{description || "This area is marked coming soon for Alpha."}</p>
    </div>
  );
}

export function Stepper({
  steps,
  current,
}: {
  steps: string[];
  current: number;
}) {
  return (
    <ol className="flex flex-wrap gap-2">
      {steps.map((step, i) => (
        <li
          key={step}
          className={cn(
            "rounded-md border px-2.5 py-1 text-xs",
            i === current
              ? "border-primary bg-primary/10 text-primary"
              : i < current
                ? "border-success/30 bg-success/10 text-success"
                : "border-border text-muted-foreground"
          )}
        >
          {i + 1}. {step}
        </li>
      ))}
    </ol>
  );
}
