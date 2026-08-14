import { Check } from "lucide-react";

const POINTS = [
  "Trigger replies from inbound events — including mock provider traffic.",
  "Keep humans in the loop: automations sit next to the shared inbox, not instead of it.",
  "Develop locally without Meta credentials, then switch the same flow to production.",
];

export function AutomationSection() {
  return (
    <section id="automations" className="mx-auto grid max-w-6xl items-center gap-12 px-6 py-20 md:grid-cols-2">
      <div>
        <p className="text-xs font-semibold uppercase tracking-[0.2em] text-violet-500">Automations</p>
        <h2 className="mt-3 text-3xl font-semibold tracking-tight">
          Native flows that reply when inbound arrives
        </h2>
        <p className="mt-3 text-sm leading-relaxed text-muted-foreground">
          Automations are first-class: inbound event, then a reply. Use the mock WhatsApp provider so you are not blocked on Meta while you build.
        </p>
        <ul className="mt-6 space-y-3">
          {POINTS.map((point) => (
            <li key={point} className="flex gap-3 text-sm">
              <span className="mt-0.5 inline-flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-violet-100 text-violet-700 dark:bg-violet-500/20 dark:text-violet-300">
                <Check className="h-3 w-3" />
              </span>
              {point}
            </li>
          ))}
        </ul>
      </div>
      <div className="relative overflow-hidden rounded-2xl border border-violet-200 bg-gradient-to-br from-violet-50 to-white p-6 dark:border-violet-500/20 dark:from-violet-500/10 dark:to-card">
        <div className="space-y-3 text-sm">
          <div className="rounded-2xl rounded-tl-md bg-violet-100 px-4 py-3 text-violet-900 dark:bg-violet-500/20 dark:text-violet-100">
            Inbound · mock provider · “I want to cancel”
          </div>
          <div className="ml-8 rounded-2xl rounded-tr-md bg-white px-4 py-3 shadow-sm ring-1 ring-violet-100 dark:bg-card dark:ring-violet-500/20">
            Automation · send template “cancel-ack” and assign to Support
          </div>
          <div className="rounded-2xl bg-primary/10 px-4 py-3 text-primary">
            Inbox · thread now in Mine · campaign paused for this contact
          </div>
        </div>
      </div>
    </section>
  );
}
