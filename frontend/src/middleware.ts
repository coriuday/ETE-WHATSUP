import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const accessToken = request.cookies.get("access_token")?.value;

  if (pathname.startsWith("/dashboard") || pathname.startsWith("/onboarding")) {
    // Cookie presence gate only — tokens are still readable client-side (XSS residual risk).
    if (!accessToken && !pathname.startsWith("/login")) {
      const loginUrl = new URL("/login", request.url);
      loginUrl.searchParams.set("next", pathname);
      return NextResponse.redirect(loginUrl);
    }
  }

  if (
    accessToken &&
    (pathname === "/login" || pathname === "/register")
  ) {
    return NextResponse.redirect(new URL("/dashboard", request.url));
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    "/dashboard/:path*",
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
    "/billing/:path*",
    "/login",
    "/register",
  ],
};
