"use client";

import { ComingSoon, PageHeader } from "@/components/ui";

export default function NotificationsPage() {
  return (
    <div className="space-y-6">
      <PageHeader title="Notifications" />
      <ComingSoon title="Notification center" />
    </div>
  );
}
