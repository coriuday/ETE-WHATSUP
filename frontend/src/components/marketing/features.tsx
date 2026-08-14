import { Send, Inbox, Users, BarChart3, ArrowRight } from "lucide-react";

const FEATURES = [
  {
    icon: Send,
    title: "Bulk messaging",
    body: "Launch campaigns from a wizard: audience, template or freeform, schedule, then send through mock or Meta.",
    href: "#campaigns",
    tone: "bg-primary/10 text-primary",
  },
  {
    icon: Inbox,
    title: "Shared inbox",
    body: "Three panes, assignment, and filters for all, unassigned, mine, and closed — one queue for the team.",
    href: "#inbox",
    tone: "bg-violet-100 text-violet-700 dark:bg-violet-500/15 dark:text-violet-300",
  },
  {
    icon: Users,
    title: "Contacts",
    body: "Lists, tags, and segments so campaigns hit the right people without a separate CRM export.",
    href: "#features",
    tone: "bg-violet-100 text-violet-700 dark:bg-violet-500/15 dark:text-violet-300",
  },
  {
    icon: BarChart3,
    title: "Analytics",
    body: "Track sent and delivered from the same workspace. No vanity dashboards pretending to be live API data.",
    href: "#features",
    tone: "bg-amber-100 text-amber-700 dark:bg-amber-500/15 dark:text-amber-300",
  },
];

export function MarketingFeatures() {
  return (
    <section id="features" className="mx-auto max-w-6xl px-6 py-20">
      <p className="text-center text-xs font-semibold uppercase tracking-[0.2em] text-primary">Powerful features</p>
      <h2 className="mx-auto mt-3 max-w-xl text-center text-3xl font-semibold tracking-tight">
        Everything you need to run WhatsApp ops{" "}
        <span className="underline decoration-primary decoration-4 underline-offset-4">in one place</span>
      </h2>
      <div className="mt-12 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
        {FEATURES.map((feature) => {
          const Icon = feature.icon;
          return (
            <article key={feature.title} className="rounded-2xl border border-border bg-card p-5 shadow-sm">
              <div className={`mb-4 inline-flex h-10 w-10 items-center justify-center rounded-xl ${feature.tone}`}>
                <Icon className="h-5 w-5" />
              </div>
              <h3 className="font-semibold tracking-tight">{feature.title}</h3>
              <p className="mt-2 text-sm leading-relaxed text-muted-foreground">{feature.body}</p>
              <a href={feature.href} className="mt-4 inline-flex items-center gap-1 text-sm font-medium text-primary">
                Learn more <ArrowRight className="h-3.5 w-3.5" />
              </a>
            </article>
          );
        })}
      </div>
    </section>
  );
}
