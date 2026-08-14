"use client";

import { useQuery } from "@tanstack/react-query";
import { api, getErrorMessage } from "@/lib/api";
import { PageHeader, DataTable, StatusBadge, ErrorState, EmptyState, Button, Input } from "@/components/ui";
import { useSearchParams } from "next/navigation";
import { useState } from "react";
import toast from "react-hot-toast";

export default function WhatsAppPage() {
  const tab = useSearchParams().get("tab") || "accounts";
  const q = useQuery({
    queryKey: ["wa-accounts"],
    queryFn: async () => (await api.get("/whatsapp/accounts")).data?.data?.accounts || [],
  });
  const [displayName, setDisplayName] = useState("Mock WhatsApp");
  const [phone, setPhone] = useState("+15550001111");

  if (q.isError) return <ErrorState onRetry={() => q.refetch()} />;
  const items = q.data || [];

  return (
    <div className="space-y-6">
      <PageHeader
        title="WhatsApp"
        description="Accounts are provider-neutral. Without Meta credentials, Alpha uses the Mock provider."
      />
      {tab === "health" && items[0] && (
        <Button
          onClick={async () => {
            try {
              const res = await api.get(`/whatsapp/accounts/${items[0].id}/health`);
              toast.success(res.data?.data?.status || "healthy");
            } catch (e) {
              toast.error(getErrorMessage(e));
            }
          }}
        >
          Run health check
        </Button>
      )}
      {!items.length && (
        <EmptyState
          title="Provider not configured"
          description="Connect WhatsApp to enable live messaging. You can add a mock account to keep campaigns and inbox working."
        />
      )}
      <div className="flex flex-wrap gap-2">
        <Input value={displayName} onChange={(e) => setDisplayName(e.target.value)} />
        <Input value={phone} onChange={(e) => setPhone(e.target.value)} />
        <Button
          onClick={async () => {
            try {
              await api.post("/whatsapp/accounts", {
                display_name: displayName,
                phone_number: phone,
                access_token: "mock",
                phone_number_id: "mock",
                waba_id: "mock",
              });
              toast.success("Account added");
              q.refetch();
            } catch (e) {
              toast.error(getErrorMessage(e));
            }
          }}
        >
          Add mock account
        </Button>
      </div>
      <DataTable headers={["Name", "Number", "Provider", "Status"]}>
        {items.map((a: { id: string; display_name: string; phone_number: string; provider?: string; status?: string }) => (
          <tr key={a.id} className="border-b border-border">
            <td className="px-3 py-2">{a.display_name}</td>
            <td className="px-3 py-2">{a.phone_number}</td>
            <td className="px-3 py-2"><StatusBadge status={a.provider || "mock"} /></td>
            <td className="px-3 py-2"><StatusBadge status={a.status || "connected"} /></td>
          </tr>
        ))}
      </DataTable>
    </div>
  );
}
