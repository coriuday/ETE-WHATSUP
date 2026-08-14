"use client";

import { ComingSoon, PageHeader } from "@/components/ui";

export default function ActivityPage() {
  return (
    <div className="space-y-6">
      <PageHeader title="Activity" />
      <ComingSoon title="Activity feed" />
    </div>
  );
}
