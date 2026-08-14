import { type NextRequest, NextResponse } from "next/server";

import { SESSION_COOKIE, verifySession } from "@/lib/proexel/auth-token";

export async function proxy(request: NextRequest) {
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
  matcher: ["/dashboard/:path*", "/api/proexel/:path*", "/auth/login"],
};
