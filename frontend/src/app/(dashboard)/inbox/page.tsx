"use client";

import { useEffect, useMemo, useState } from "react";
import { usePathname } from "next/navigation";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import toast from "react-hot-toast";
import {
  SearchInput,
  Button,
  EmptyState,
  ErrorState,
  LoadingSkeleton,
  StatusBadge,
  Tag,
  Textarea,
} from "@/components/ui";
import {
  addNote,
  getConversation,
  getConversationMessages,
  listConversations,
  listNotes,
  reopenConversation,
  resolveConversation,
  sendConversationMessage,
} from "@/lib/api/inbox";
import { api } from "@/lib/api";
import { cn, formatDateTime } from "@/lib/utils";

export default function InboxPage() {
  const pathname = usePathname();
  const qc = useQueryClient();
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [compose, setCompose] = useState("");
  const [note, setNote] = useState("");

  const filter = useMemo(() => {
    const params: Record<string, unknown> = { search };
    if (pathname.endsWith("/unassigned")) params.unassigned = true;
    if (pathname.endsWith("/mine")) params.mine = true;
    if (pathname.endsWith("/closed")) params.status = "resolved";
    return params;
  }, [pathname, search]);

  const list = useQuery({
    queryKey: ["conversations", filter],
    queryFn: () => listConversations(filter),
    refetchInterval: 5000,
  });

  const conversations = list.data?.conversations || [];
  useEffect(() => {
    if (!selectedId && conversations[0]?.id) setSelectedId(conversations[0].id);
  }, [conversations, selectedId]);

  const detail = useQuery({
    queryKey: ["conversation", selectedId],
    queryFn: () => getConversation(selectedId!),
    enabled: !!selectedId,
  });
  const messages = useQuery({
    queryKey: ["conversation-messages", selectedId],
    queryFn: () => getConversationMessages(selectedId!),
    enabled: !!selectedId,
    refetchInterval: 4000,
  });
  const notes = useQuery({
    queryKey: ["conversation-notes", selectedId],
    queryFn: () => listNotes(selectedId!),
    enabled: !!selectedId,
  });
  const templates = useQuery({
    queryKey: ["templates-inbox"],
    queryFn: async () => (await api.get("/templates")).data?.data,
  });

  const send = useMutation({
    mutationFn: () =>
      sendConversationMessage(selectedId!, { body: compose, message_type: "text" }),
    onSuccess: () => {
      setCompose("");
      qc.invalidateQueries({ queryKey: ["conversation-messages", selectedId] });
      qc.invalidateQueries({ queryKey: ["conversations"] });
    },
    onError: () => toast.error("Message failed to send"),
  });

  if (list.isError) {
    return <ErrorState message="Inbox failed to load" onRetry={() => list.refetch()} />;
  }

  return (
    <div className="grid h-full grid-cols-1 overflow-hidden lg:grid-cols-[280px_1fr_280px]">
      <section className="flex min-h-0 flex-col border-r border-border bg-card">
        <div className="border-b border-border p-3">
          <SearchInput placeholder="Search conversations" value={search} onChange={(e) => setSearch(e.target.value)} />
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto">
          {list.isLoading ? (
            <div className="space-y-2 p-3">
              <LoadingSkeleton className="h-14" />
              <LoadingSkeleton className="h-14" />
            </div>
          ) : !conversations.length ? (
            <EmptyState title="No conversations" description="Simulate an inbound message from Developer settings or wait for contacts to reply." />
          ) : (
            conversations.map((c: { id: string; unread_count?: number; last_message_body?: string; contact?: { name?: string; phone?: string } }) => (
              <button
                key={c.id}
                onClick={() => setSelectedId(c.id)}
                className={cn(
                  "w-full border-b border-border px-3 py-3 text-left",
                  selectedId === c.id ? "bg-primary/5" : "hover:bg-accent"
                )}
              >
                <div className="flex items-center justify-between">
                  <p className="text-sm font-medium">{c.contact?.name || c.contact?.phone}</p>
                  {!!c.unread_count && (
                    <span className="rounded-full bg-primary px-1.5 text-[10px] text-primary-foreground">{c.unread_count}</span>
                  )}
                </div>
                <p className="truncate text-xs text-muted-foreground">{c.last_message_body}</p>
              </button>
            ))
          )}
        </div>
      </section>

      <section className="flex min-h-0 flex-col bg-background">
        {!selectedId ? (
          <EmptyState title="Select a conversation" />
        ) : (
          <>
            <header className="flex items-center justify-between border-b border-border px-4 py-3">
              <div>
                <p className="font-medium">{detail.data?.contact?.first_name || detail.data?.contact?.phone}</p>
                <p className="text-xs text-muted-foreground">{detail.data?.contact?.phone}</p>
              </div>
              <div className="flex gap-2">
                <Button size="sm" variant="outline" onClick={() => resolveConversation(selectedId!).then(() => qc.invalidateQueries())}>
                  Close
                </Button>
                <Button size="sm" variant="ghost" onClick={() => reopenConversation(selectedId!).then(() => qc.invalidateQueries())}>
                  Reopen
                </Button>
              </div>
            </header>
            <div className="min-h-0 flex-1 space-y-3 overflow-y-auto p-4">
              {(messages.data?.messages || []).map((m: { id: string; direction: string; body?: string; status?: string; created_at: string }) => (
                <div key={m.id} className={cn("max-w-[75%] rounded-xl border px-3 py-2 text-sm", m.direction === "outbound" ? "ml-auto border-primary/20 bg-primary/5" : "border-border bg-card")}>
                  <p>{m.body}</p>
                  <p className="mt-1 flex items-center gap-2 text-[10px] text-muted-foreground">
                    {formatDateTime(m.created_at)}
                    {m.direction === "outbound" && <StatusBadge status={m.status || "sent"} />}
                  </p>
                </div>
              ))}
            </div>
            <footer className="border-t border-border p-3">
              <div className="mb-2 flex flex-wrap gap-1">
                {(templates.data?.templates || templates.data || []).slice?.(0, 4)?.map?.((t: { id: string; name: string }) => (
                  <button key={t.id} className="rounded-md border border-border px-2 py-1 text-xs" onClick={() => setCompose((p) => p || t.name)}>
                    {t.name}
                  </button>
                ))}
              </div>
              <div className="flex gap-2">
                <Textarea value={compose} onChange={(e) => setCompose(e.target.value)} placeholder="Write a reply" className="min-h-[72px]" />
                <Button disabled={!compose.trim() || send.isPending} onClick={() => send.mutate()}>
                  Send
                </Button>
              </div>
            </footer>
          </>
        )}
      </section>

      <aside className="hidden min-h-0 overflow-y-auto border-l border-border bg-card p-4 lg:block">
        <h3 className="text-sm font-semibold">Contact</h3>
        <p className="mt-2 text-sm">{detail.data?.contact?.phone}</p>
        <div className="mt-3 flex flex-wrap gap-1">
          {(detail.data?.contact?.tags || []).map((t: string) => (
            <Tag key={t}>{t}</Tag>
          ))}
        </div>
        <h4 className="mt-6 text-xs font-semibold uppercase text-muted-foreground">Notes</h4>
        <div className="mt-2 space-y-2">
          {(notes.data?.notes || []).map((n: { id: string; body: string }) => (
            <p key={n.id} className="rounded-md bg-muted px-2 py-1 text-xs">{n.body}</p>
          ))}
        </div>
        <Textarea className="mt-2 min-h-[64px]" value={note} onChange={(e) => setNote(e.target.value)} placeholder="Add a note" />
        <Button
          className="mt-2"
          size="sm"
          variant="outline"
          onClick={() => selectedId && addNote(selectedId, note).then(() => { setNote(""); qc.invalidateQueries({ queryKey: ["conversation-notes", selectedId] }); })}
        >
          Save note
        </Button>
      </aside>
    </div>
  );
}
