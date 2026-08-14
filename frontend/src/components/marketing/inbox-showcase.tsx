export function InboxShowcase() {
  return (
    <section id="inbox" className="bg-[#042F2E] py-20 text-white">
      <div className="mx-auto max-w-6xl px-6">
        <p className="text-xs font-semibold uppercase tracking-[0.2em] text-teal-300">Inbox</p>
        <h2 className="mt-3 max-w-xl text-3xl font-semibold tracking-tight">
          All your conversations in one shared inbox
        </h2>
        <p className="mt-3 max-w-xl text-sm leading-relaxed text-white/70">
          Assign threads, filter unassigned and mine, and keep closed conversations out of the live queue.
        </p>
        <div className="mt-10 overflow-hidden rounded-2xl border border-white/10 bg-[#0A3D3A] shadow-2xl">
          <div className="grid min-h-[320px] grid-cols-1 text-[12px] md:grid-cols-[220px_1fr_200px]">
            <div className="space-y-1 border-b border-white/10 p-3 md:border-b-0 md:border-r">
              <p className="mb-2 px-2 text-[10px] font-semibold uppercase tracking-wider text-white/40">Conversations</p>
              {[
                { name: "Priya · order delay", preview: "Where is my order?", active: true },
                { name: "Alex · template reply", preview: "Thanks — received." },
                { name: "Ops · mock inbound", preview: "Mock provider ping" },
              ].map((row) => (
                <div
                  key={row.name}
                  className={`rounded-xl px-3 py-2.5 ${row.active ? "bg-teal-500/20 text-white" : "text-white/70"}`}
                >
                  <p className="font-medium">{row.name}</p>
                  <p className="text-white/50">{row.preview}</p>
                </div>
              ))}
            </div>
            <div className="flex flex-col border-b border-white/10 p-4 md:border-b-0 md:border-r">
              <p className="mb-4 text-sm font-medium">Priya Sharma</p>
              <div className="mt-auto space-y-2">
                <div className="max-w-[80%] rounded-2xl rounded-tl-md bg-white/10 px-3 py-2 text-white/90">
                  Where is my order? Tracking still shows packed.
                </div>
                <div className="ml-auto max-w-[80%] rounded-2xl rounded-tr-md bg-teal-500/90 px-3 py-2 text-white">
                  On it — tracking update sent from the inbox.
                </div>
                <div className="mt-3 rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-white/40">
                  Reply in this thread…
                </div>
              </div>
            </div>
            <div className="space-y-3 p-4 text-white/70">
              <p className="text-[10px] font-semibold uppercase tracking-wider text-white/40">Contact</p>
              <p className="font-medium text-white">Priya Sharma</p>
              <p>Assigned · you</p>
              <p>Tags · orders</p>
              <p className="rounded-lg bg-teal-500/15 px-2 py-1.5 text-teal-200">Provider · Mock WhatsApp</p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
