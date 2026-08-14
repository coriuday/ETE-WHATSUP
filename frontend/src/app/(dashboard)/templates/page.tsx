"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { PageHeader, DataTable, StatusBadge, ErrorState, EmptyState, Button } from "@/components/ui";
import { useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";
import toast from "react-hot-toast";
import { getErrorMessage } from "@/lib/api";

export default function TemplatesPage() {
  return (
    <Suspense fallback={null}>
      <TemplatesPageInner />
    </Suspense>
  );
}

function TemplatesPageInner() {
  const tab = useSearchParams().get("tab") || "templates";
  const q = useQuery({
    queryKey: ["templates"],
    queryFn: async () => (await api.get("/templates")).data?.data,
  });
  const [name, setName] = useState("");
  const [body, setBody] = useState("");
  const items = q.data?.templates || q.data || [];
  if (q.isError) return <ErrorState onRetry={() => q.refetch()} />;

  return (
    <div className="space-y-6">
      <PageHeader title={tab === "quick-replies" ? "Quick replies" : "WhatsApp templates"} />
      <div className="flex gap-2">
        <input className="h-9 rounded-lg border border-input px-3 text-sm" placeholder="Name" value={name} onChange={(e) => setName(e.target.value)} />
        <input className="h-9 flex-1 rounded-lg border border-input px-3 text-sm" placeholder="Body" value={body} onChange={(e) => setBody(e.target.value)} />
        <Button
          onClick={async () => {
            try {
              await api.post("/templates", { name, display_name: name, body_text: body, category: "utility", language: "en" });
              toast.success("Created");
              q.refetch();
            } catch (e) {
              toast.error(getErrorMessage(e));
            }
          }}
        >
          Create
        </Button>
      </div>
      {!items.length ? (
        <EmptyState title="No templates" />
      ) : (
        <DataTable headers={["Name", "Status", "Language"]}>
          {(Array.isArray(items) ? items : []).map((t: { id: string; name: string; status?: string; language?: string }) => (
            <tr key={t.id} className="border-b border-border">
              <td className="px-3 py-2">{t.name}</td>
              <td className="px-3 py-2"><StatusBadge status={t.status || "draft"} /></td>
              <td className="px-3 py-2">{t.language}</td>
            </tr>
          ))}
        </DataTable>
      )}
    </div>
  );
}
