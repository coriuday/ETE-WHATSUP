import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import Login from "@/app/(auth)/login/page";

vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn() }),
}));

vi.mock("next/link", () => ({
  default: ({ children, href }: { children: React.ReactNode; href: string }) => (
    <a href={href}>{children}</a>
  ),
}));

vi.mock("@/store/authStore", () => ({
  useAuthStore: () => ({
    setTokens: vi.fn(),
    setUser: vi.fn(),
    setOrganization: vi.fn(),
  }),
}));

describe("login page", () => {
  it("renders shadcn sign-in form", () => {
    render(<Login />);
    expect(screen.getByRole("heading", { name: "Sign in" })).toBeTruthy();
    expect(screen.getByLabelText("Email")).toBeTruthy();
    expect(screen.getByLabelText("Password")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sign in" })).toBeTruthy();
  });
});
