export function HeroProductMock() {
  return (
    <div className="relative mx-auto w-full max-w-[520px]">
      <div className="rounded-2xl border border-border bg-card p-4 shadow-xl">
        <div className="mb-3 flex items-center justify-between text-[11px] text-muted-foreground">
          <span className="font-medium text-foreground">Campaigns · ChatBridge</span>
          <span className="rounded-full bg-primary/10 px-2 py-0.5 text-primary">Mock WhatsApp</span>
        </div>
        <div className="mb-4 grid grid-cols-2 gap-2">
          <div className="rounded-xl bg-muted p-3">
            <p className="text-[10px] uppercase tracking-wider text-muted-foreground">Queue</p>
            <p className="text-lg font-semibold text-primary">Inbox</p>
            <p className="text-[11px] text-muted-foreground">All · Mine · Closed</p>
          </div>
          <div className="rounded-xl bg-muted p-3">
            <p className="text-[10px] uppercase tracking-wider text-muted-foreground">Last send</p>
            <p className="text-lg font-semibold">Wizard</p>
            <p className="text-[11px] text-muted-foreground">Audience → review</p>
          </div>
        </div>
        <div className="h-16 rounded-xl bg-gradient-to-r from-primary/20 to-primary/5" />
      </div>

      <div className="absolute -bottom-8 -right-2 w-[200px] rounded-[1.75rem] border-4 border-[#042F2E] bg-[#042F2E] p-2 shadow-2xl sm:right-4">
        <div className="rounded-[1.25rem] bg-[#0F3D3A] p-3 text-[10px] text-white">
          <p className="mb-2 text-center text-[9px] text-white/50">WhatsApp · mock</p>
          <div className="mb-1.5 max-w-[90%] rounded-lg rounded-tl-sm bg-white/10 px-2 py-1.5">Flash sale window is open — reply STOP to opt out.</div>
          <div className="ml-auto max-w-[85%] rounded-lg rounded-tr-sm bg-primary px-2 py-1.5">Campaign queued. Inbox will catch replies.</div>
        </div>
      </div>
    </div>
  );
}

export function ProductFrame() {
  return (
    <div className="overflow-hidden rounded-xl border border-border bg-card">
      <div className="flex items-center gap-2 border-b border-border px-3 py-2">
        <span className="h-2 w-2 rounded-full bg-border" />
        <span className="h-2 w-2 rounded-full bg-border" />
        <span className="h-2 w-2 rounded-full bg-border" />
        <span className="ml-2 text-[11px] text-muted-foreground">Inbox · ChatBridge</span>
      </div>
      <div className="grid min-h-[280px] grid-cols-[88px_1fr_104px] text-[11px]">
        <div className="space-y-1.5 border-r border-border bg-muted/40 p-2">
          {["All", "Mine", "Closed"].map((item, i) => (
            <div
              key={item}
              className={`rounded-md px-2 py-1.5 ${i === 0 ? "bg-primary/10 font-medium text-primary" : "text-muted-foreground"}`}
            >
              {item}
            </div>
          ))}
        </div>
        <div className="space-y-2 p-2">
          {["Priya · order delay", "Alex · template reply", "Ops · mock inbound"].map((row, i) => (
            <div key={row} className={`rounded-md border border-border px-2 py-2 ${i === 0 ? "bg-accent" : ""}`}>
              <p className="font-medium text-foreground">{row}</p>
              <p className="text-muted-foreground">Last message · 2m</p>
            </div>
          ))}
        </div>
        <div className="space-y-2 border-l border-border p-2 text-muted-foreground">
          <p className="font-medium text-foreground">Thread</p>
          <div className="rounded-md bg-muted px-2 py-1.5">Where is my order?</div>
          <div className="rounded-md bg-primary/10 px-2 py-1.5 text-primary">On it — tracking sent.</div>
        </div>
      </div>
    </div>
  );
}

export function CampaignFrame() {
  return (
    <div className="overflow-hidden rounded-2xl border border-border bg-card shadow-sm">
      <div className="border-b border-border px-4 py-2 text-xs font-medium">Campaign wizard</div>
      <div className="space-y-3 p-4 text-sm">
        {["Audience · tagged contacts", "Message · Hello {{first_name}}", "Provider · Mock WhatsApp"].map((line) => (
          <div key={line} className="rounded-xl border border-border px-3 py-2">
            {line}
          </div>
        ))}
      </div>
    </div>
  );
}
