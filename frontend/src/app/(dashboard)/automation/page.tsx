"use client";

import { Suspense, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useSearchParams } from "next/navigation";
import { api, getErrorMessage } from "@/lib/api";
import { PageHeader, Button, Input, DataTable, StatusBadge, ErrorState, EmptyState, NativeSelect } from "@/components/ui";
import toast from "react-hot-toast";

export default function AutomationPage() {
  return (
    <Suspense fallback={null}>
      <AutomationPageInner />
    </Suspense>
  );
}

function AutomationPageInner() {
  const tab = useSearchParams().get("tab") || "flows";
  const flows = useQuery({
    queryKey: ["automations"],
    queryFn: async () => (await api.get("/automations")).data?.data?.workflows || [],
  });
  const runs = useQuery({
    queryKey: ["automation-runs"],
    queryFn: async () => (await api.get("/automations/runs")).data?.data?.runs || [],
  });
  const [name, setName] = useState("Welcome tag");
  const [trigger, setTrigger] = useState("message.received");

  if (flows.isError) return <ErrorState onRetry={() => flows.refetch()} />;

  return (
    <div className="space-y-6">
      <PageHeader title="Automations" description="Native workflows. n8n is an optional action, not the engine." />
      {tab !== "logs" && (
        <div className="flex flex-wrap gap-2">
          <Input value={name} onChange={(e) => setName(e.target.value)} />
          <NativeSelect value={trigger} onChange={(e) => setTrigger(e.target.value)}>
            <option value="message.received">Message received</option>
            <option value="contact.created">Contact created</option>
            <option value="campaign.completed">Campaign completed</option>
            <option value="tag.added">Tag added</option>
          </NativeSelect>
          <Button
            onClick={async () => {
              try {
                await api.post("/automations", {
                  name,
                  trigger_type: trigger,
                  definition: { steps: [{ type: "add_tag", tag: "engaged" }] },
                });
                toast.success("Flow created");
                flows.refetch();
              } catch (e) {
                toast.error(getErrorMessage(e));
              }
            }}
          >
            Create flow
          </Button>
        </div>
      )}
      {tab === "logs" ? (
        !runs.data?.length ? <EmptyState title="No runs yet" /> : (
          <DataTable headers={["Trigger", "Status", "Started"]}>
            {runs.data.map((r: { id: string; trigger_type: string; status: string; started_at: string }) => (
              <tr key={r.id} className="border-b border-border">
                <td className="px-3 py-2">{r.trigger_type}</td>
                <td className="px-3 py-2"><StatusBadge status={r.status} /></td>
                <td className="px-3 py-2">{r.started_at}</td>
              </tr>
            ))}
          </DataTable>
        )
      ) : !flows.data?.length ? (
        <EmptyState title="No flows" />
      ) : (
        <DataTable headers={["Name", "Trigger", "Enabled"]}>
          {flows.data.map((f: { id: string; name: string; trigger_type: string; enabled: boolean }) => (
            <tr key={f.id} className="border-b border-border">
              <td className="px-3 py-2">{f.name}</td>
              <td className="px-3 py-2">{f.trigger_type}</td>
              <td className="px-3 py-2">{f.enabled ? "Yes" : "No"}</td>
            </tr>
          ))}
        </DataTable>
      )}
    </div>
  );
}
