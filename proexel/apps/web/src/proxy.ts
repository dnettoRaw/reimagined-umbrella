import { type NextRequest, NextResponse } from "next/server";

import { SESSION_COOKIE, verifySession } from "@/lib/proexel/auth-token";
import {
  createDemoAccessToken,
  DEMO_ACCESS_COOKIE,
  DEMO_PROFILE_COOKIE,
  findDemoProfile,
  getDemoPassword,
} from "@/lib/proexel/demo-access";

function safeDestination(value: string | null) {
  return value?.startsWith("/") && !value.startsWith("//") ? value : "/dashboard/overview";
}

export async function proxy(request: NextRequest) {
  if (process.env.PROEXEL_DEMO === "1" || process.env.NEXT_PUBLIC_PROEXEL_DEMO === "1") {
    const path = request.nextUrl.pathname;
    if (path === "/api/demo-access") return NextResponse.next();

    const expectedToken = await createDemoAccessToken(getDemoPassword());
    const hasAccess = request.cookies.get(DEMO_ACCESS_COOKIE)?.value === expectedToken;
    const profile = findDemoProfile(request.cookies.get(DEMO_PROFILE_COOKIE)?.value);
    if (path === "/demo-access") {
      if (!hasAccess || !profile) return NextResponse.next();
      return NextResponse.redirect(new URL(safeDestination(request.nextUrl.searchParams.get("next")), request.url));
    }
    if (!hasAccess || !profile) {
      if (path.startsWith("/api/")) {
        return NextResponse.json(
          { error: hasAccess ? "demo_profile_required" : "demo_access_required" },
          { status: 401 },
        );
      }
      const access = new URL("/demo-access", request.url);
      access.searchParams.set("next", `${path}${request.nextUrl.search}`);
      return NextResponse.redirect(access);
    }
    if (request.nextUrl.pathname === "/auth/login") {
      return NextResponse.redirect(new URL("/dashboard/overview", request.url));
    }
    return NextResponse.next();
  }
  const session = await verifySession(request.cookies.get(SESSION_COOKIE)?.value, process.env.PROEXEL_SESSION_SECRET);
  const path = request.nextUrl.pathname;
  if (path === "/auth/login" && session) {
    return NextResponse.redirect(new URL("/dashboard/overview", request.url));
  }
  if ((path.startsWith("/dashboard") || path.startsWith("/api/proexel")) && !session) {
    if (path.startsWith("/api/")) {
      return NextResponse.json({ error: "unauthenticated" }, { status: 401 });
    }
    const login = new URL("/auth/login", request.url);
    login.searchParams.set("next", path);
    return NextResponse.redirect(login);
  }
  return NextResponse.next();
}

export const config = {
  matcher: ["/dashboard/:path*", "/api/proexel/:path*", "/api/demo-access", "/auth/login", "/demo-access"],
};
