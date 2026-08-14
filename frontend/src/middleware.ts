import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

const PROTECTED_PREFIXES = [
  "/dashboard",
  "/activity",
  "/notifications",
  "/onboarding",
  "/contacts",
  "/campaigns",
  "/inbox",
  "/templates",
  "/whatsapp",
  "/team",
  "/settings",
  "/schedules",
  "/automation",
  "/automations",
  "/billing",
  "/analytics",
  "/integrations",
];

export function middleware(request: NextRequest) {
  // `/`, marketing, and auth recovery pages stay public (not in `matcher`).
  const { pathname } = request.nextUrl;
  const accessToken = request.cookies.get("access_token")?.value;

  const isProtected = PROTECTED_PREFIXES.some(
    (p) => pathname === p || pathname.startsWith(`${p}/`)
  );

  if (isProtected && !accessToken) {
    const loginUrl = new URL("/login", request.url);
    loginUrl.searchParams.set("next", pathname);
    return NextResponse.redirect(loginUrl);
  }

  if (accessToken && (pathname === "/login" || pathname === "/register")) {
    return NextResponse.redirect(new URL("/dashboard", request.url));
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    "/dashboard/:path*",
    "/activity/:path*",
    "/notifications/:path*",
    "/onboarding",
    "/contacts/:path*",
    "/campaigns/:path*",
    "/inbox/:path*",
    "/templates/:path*",
    "/whatsapp/:path*",
    "/team/:path*",
    "/settings/:path*",
    "/schedules/:path*",
    "/automation/:path*",
    "/automations/:path*",
    "/billing/:path*",
    "/analytics/:path*",
    "/integrations/:path*",
    "/login",
    "/register",
  ],
};
