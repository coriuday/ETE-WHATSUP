import { describe, expect, it } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { EmptyState } from "@/components/ui/empty-state";
import { ErrorState } from "@/components/ui/error-state";
import { Stepper } from "@/components/ui/stat-card";

describe("design system states", () => {
  it("renders empty state copy", () => {
    const html = renderToStaticMarkup(
      <EmptyState title="No conversations" description="Wait for replies." />
    );
    expect(html).toContain("No conversations");
  });

  it("renders error state", () => {
    const html = renderToStaticMarkup(<ErrorState message="Inbox failed" />);
    expect(html).toContain("Inbox failed");
  });

  it("renders campaign wizard steps", () => {
    const html = renderToStaticMarkup(
      <Stepper steps={["Campaign", "Audience", "Message"]} current={0} />
    );
    expect(html).toContain("Campaign");
    expect(html).toContain("Audience");
  });
});
