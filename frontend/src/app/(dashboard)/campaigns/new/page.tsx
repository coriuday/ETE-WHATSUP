"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useQuery } from "@tanstack/react-query";
import toast from "react-hot-toast";
import { DateTimePicker } from "@mui/x-date-pickers/DateTimePicker";
import { PageHeader, Button, Input, Textarea, NativeSelect, Stepper, StatCard } from "@/components/ui";
import { api, getErrorMessage } from "@/lib/api";

const STEPS = ["Campaign", "Audience", "Message", "Personalization", "Schedule", "Review", "Launch"];

export default function NewCampaignPage() {
  const router = useRouter();
  const [step, setStep] = useState(0);
  const [name, setName] = useState("");
  const [accountId, setAccountId] = useState("");
  const [targetType, setTargetType] = useState("all_contacts");
  const [segmentId, setSegmentId] = useState("");
  const [messageBody, setMessageBody] = useState("Hello {{first_name}}, this is WhatsUp.");
  const [templateId, setTemplateId] = useState("");
  const [mappings, setMappings] = useState<Record<string, string>>({});
  const [scheduleAt, setScheduleAt] = useState<Date | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const accounts = useQuery({
    queryKey: ["wa-accounts"],
    queryFn: async () => (await api.get("/whatsapp/accounts")).data?.data?.accounts || [],
  });
  const templates = useQuery({
    queryKey: ["templates"],
    queryFn: async () => (await api.get("/templates")).data?.data?.templates || (await api.get("/templates")).data?.data || [],
  });
  const segments = useQuery({
    queryKey: ["segments"],
    queryFn: async () => (await api.get("/contacts/segments")).data?.data?.segments || [],
  });
  const contactCount = useQuery({
    queryKey: ["contacts-count"],
    queryFn: async () => {
      const res = await api.get("/contacts", { params: { limit: 1 } });
      return res.data?.data?.pagination?.total ?? 0;
    },
  });

  const selectedAccount = (accounts.data || []).find((a: { id: string }) => a.id === accountId);

  async function launch() {
    setSubmitting(true);
    try {
      const wa = accountId || accounts.data?.[0]?.id;
      if (!wa) {
        toast.error("Create a WhatsApp account (mock is fine) first");
        return;
      }
      const created = await api.post("/campaigns", {
        name,
        type: "bulk_message",
        wa_account_id: wa,
        target_type: targetType,
        target_segment_id: targetType === "segment" ? segmentId || null : null,
        message_body: messageBody,
        template_id: templateId || null,
        buttons: mappings,
      });
      const id = created.data?.data?.id;
      if (scheduleAt) {
        await api.post(`/campaigns/${id}/schedule`, { scheduled_at: new Date(scheduleAt).toISOString() });
        toast.success("Campaign scheduled");
      } else {
        await api.post(`/campaigns/${id}/launch`);
        toast.success("Campaign launched");
      }
      router.push(`/campaigns/${id}`);
    } catch (e) {
      toast.error(getErrorMessage(e));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="space-y-6">
      <PageHeader title="Create campaign" description="Wizard-based campaign setup." />
      <Stepper steps={STEPS} current={step} />
      <div className="rounded-xl border border-border bg-card p-6 space-y-4">
        {step === 0 && (
          <>
            <Input placeholder="Campaign name" value={name} onChange={(e) => setName(e.target.value)} />
            <NativeSelect value={accountId} onChange={(e) => setAccountId(e.target.value)}>
              <option value="">Select provider account</option>
              {(accounts.data || []).map((a: { id: string; display_name: string; provider?: string }) => (
                <option key={a.id} value={a.id}>{a.display_name} ({a.provider || "mock"})</option>
              ))}
            </NativeSelect>
          </>
        )}
        {step === 1 && (
          <>
            <NativeSelect value={targetType} onChange={(e) => setTargetType(e.target.value)}>
              <option value="all_contacts">All contacts</option>
              <option value="segment">Segment</option>
              <option value="group">List / group</option>
              <option value="custom_list">Saved / custom list</option>
            </NativeSelect>
            {targetType === "segment" && (
              <NativeSelect value={segmentId} onChange={(e) => setSegmentId(e.target.value)}>
                <option value="">Select segment</option>
                {(segments.data || []).map((s: { id: string; name: string }) => (
                  <option key={s.id} value={s.id}>{s.name}</option>
                ))}
              </NativeSelect>
            )}
            <p className="text-sm text-muted-foreground">Estimated audience: {contactCount.data ?? "—"} contacts</p>
          </>
        )}
        {step === 2 && (
          <>
            <NativeSelect value={templateId} onChange={(e) => setTemplateId(e.target.value)}>
              <option value="">Freeform text</option>
              {(Array.isArray(templates.data) ? templates.data : []).map((t: { id: string; name: string }) => (
                <option key={t.id} value={t.id}>{t.name}</option>
              ))}
            </NativeSelect>
            <Textarea value={messageBody} onChange={(e) => setMessageBody(e.target.value)} />
            <p className="text-xs text-muted-foreground">Preview: {messageBody.replaceAll("{{first_name}}", "Alex")}</p>
          </>
        )}
        {step === 3 && (
          <Input placeholder="{{1}} mapping e.g. first_name" onChange={(e) => setMappings({ "1": e.target.value })} />
        )}
        {step === 4 && (
          <DateTimePicker
            label="Schedule (optional)"
            value={scheduleAt}
            onChange={(value) => setScheduleAt(value)}
            slotProps={{ textField: { size: "small", fullWidth: true } }}
          />
        )}
        {step === 5 && (
          <div className="grid gap-3 sm:grid-cols-2">
            <StatCard label="Audience" value={contactCount.data ?? 0} />
            <StatCard label="Messages" value={contactCount.data ?? 0} hint="One per contact" />
            <StatCard label="Schedule" value={scheduleAt ? scheduleAt.toLocaleString() : "Immediate"} />
            <StatCard label="Provider" value={selectedAccount?.provider || selectedAccount?.display_name || "Mock"} />
          </div>
        )}
        {step === 6 && (
          <p className="text-sm text-muted-foreground">Ready to {scheduleAt ? "schedule" : "launch"} “{name}”.</p>
        )}
        <div className="flex justify-between pt-4">
          <Button variant="outline" disabled={step === 0} onClick={() => setStep(step - 1)}>Back</Button>
          {step < 6 ? (
            <Button onClick={() => setStep(step + 1)} disabled={step === 0 && !name}>Next</Button>
          ) : (
            <Button onClick={launch} disabled={submitting}>{scheduleAt ? "Schedule" : "Launch"}</Button>
          )}
        </div>
      </div>
    </div>
  );
}
