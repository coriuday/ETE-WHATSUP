const QUOTES = [
  {
    quote: "We finally have one queue instead of a spreadsheet of unread WhatsApp threads.",
    name: "Ops lead",
    role: "Support team",
  },
  {
    quote: "Campaign wizard plus mock provider meant we could rehearse sends before Meta was approved.",
    name: "Growth manager",
    role: "E-commerce",
  },
  {
    quote: "Automations fire on inbound without replacing the humans who still own the hard cases.",
    name: "CX manager",
    role: "SaaS",
  },
];

export function Testimonials() {
  return (
    <section className="border-y border-border bg-muted/40 py-20">
      <div className="mx-auto max-w-6xl px-6">
        <h2 className="text-center text-3xl font-semibold tracking-tight">Built the way teams actually work</h2>
        <p className="mx-auto mt-3 max-w-lg text-center text-sm text-muted-foreground">
          Illustrative quotes — not customer metrics or live API numbers.
        </p>
        <div className="mt-10 grid gap-5 md:grid-cols-3">
          {QUOTES.map((item) => (
            <blockquote key={item.name} className="rounded-2xl border border-border bg-card p-6 shadow-sm">
              <p className="text-sm leading-relaxed">&ldquo;{item.quote}&rdquo;</p>
              <footer className="mt-4 text-sm">
                <p className="font-semibold">{item.name}</p>
                <p className="text-muted-foreground">{item.role}</p>
              </footer>
            </blockquote>
          ))}
        </div>
      </div>
    </section>
  );
}
