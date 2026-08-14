const STEPS = [
  { n: "1", title: "Sign up", body: "Create a workspace. No Meta credentials required to start." },
  { n: "2", title: "Connect WhatsApp", body: "Use the mock provider locally, or attach a Meta account when you are ready." },
  { n: "3", title: "Add contacts", body: "Import, tag, and segment the people you message." },
  { n: "4", title: "Inbox & campaigns", body: "Assign threads or send a bulk campaign from the wizard." },
  { n: "5", title: "Automate & track", body: "Trigger replies on inbound events and watch sent vs delivered." },
];

export function HowItWorks() {
  return (
    <section id="how-it-works" className="border-y border-border bg-muted/50 py-20">
      <div className="mx-auto max-w-6xl px-6">
        <h2 className="text-center text-3xl font-semibold tracking-tight">Get started in five steps</h2>
        <p className="mx-auto mt-3 max-w-lg text-center text-sm text-muted-foreground">
          Product-shaped onboarding — inbox, campaigns, automations, and a mock WhatsApp provider.
        </p>
        <ol className="mt-12 grid gap-8 sm:grid-cols-2 lg:grid-cols-5">
          {STEPS.map((step) => (
            <li key={step.n} className="text-center">
              <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-primary text-sm font-semibold text-primary-foreground">
                {step.n}
              </div>
              <h3 className="mt-4 text-sm font-semibold">{step.title}</h3>
              <p className="mt-2 text-xs leading-relaxed text-muted-foreground">{step.body}</p>
            </li>
          ))}
        </ol>
      </div>
    </section>
  );
}
