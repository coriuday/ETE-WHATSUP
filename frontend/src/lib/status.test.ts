import { describe, expect, it } from "vitest";
import { canTransitionCopy } from "./status";

describe("message status display", () => {
  it("does not invent states on the client", () => {
    expect(canTransitionCopy("sent")).toBe("sent");
    expect(canTransitionCopy("delivered")).toBe("delivered");
  });
});
