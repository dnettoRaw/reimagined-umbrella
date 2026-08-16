import { type NextRequest, NextResponse } from "next/server";

import { SESSION_COOKIE } from "@/lib/proexel/auth-token";
import { DEMO_PROFILE_COOKIE } from "@/lib/proexel/demo-access";
import { isDemoMode } from "@/lib/proexel/demo-service";

export async function POST(request: NextRequest) {
  const response = NextResponse.json({ ok: true });
  response.cookies.set(SESSION_COOKIE, "", {
    httpOnly: true,
    sameSite: "strict",
    secure: process.env.NODE_ENV === "production",
    path: "/",
    maxAge: 0,
  });
  if (isDemoMode()) {
    response.cookies.set(DEMO_PROFILE_COOKIE, "", {
      httpOnly: true,
      sameSite: "lax",
      secure: request.nextUrl.protocol === "https:" || request.headers.get("x-forwarded-proto") === "https",
      path: "/",
      maxAge: 0,
    });
  }
  return response;
}
