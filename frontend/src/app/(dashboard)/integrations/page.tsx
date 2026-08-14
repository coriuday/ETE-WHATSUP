"use client";

import { PageHeader, ComingSoon } from "@/components/ui";
import { useSearchParams } from "next/navigation";

export default function IntegrationsPage() {
  const tab = useSearchParams().get("tab");
  return (
    <div className="space-y-6">
      <PageHeader title="Integrations" description="n8n is an orchestration layer. Native automations live under Automations." />
      {tab === "webhooks" ? (
        <ComingSoon title="Inbound webhooks" description="Use Mock provider simulation under Settings → Developer for Alpha." />
      ) : (
        <div className="rounded-xl border border-border bg-card p-6 text-sm text-muted-foreground">
          Point n8n webhook URLs at automation_triggers in the database. Native flows can include an HTTP/n8n action.
        </div>
      )}
    </div>
  );
}
