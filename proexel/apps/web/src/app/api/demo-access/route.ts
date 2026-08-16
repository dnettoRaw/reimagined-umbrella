import { type NextRequest, NextResponse } from "next/server";

import {
  createDemoAccessToken,
  DEMO_ACCESS_COOKIE,
  DEMO_PROFILE_COOKIE,
  DEMO_PROFILES,
  findDemoProfile,
  getDemoPassword,
} from "@/lib/proexel/demo-access";

const COOKIE_MAX_AGE = 60 * 60 * 24 * 7;

function usesHttps(request: NextRequest) {
  return request.nextUrl.protocol === "https:" || request.headers.get("x-forwarded-proto") === "https";
}

export async function POST(request: NextRequest) {
  const payload = (await request.json().catch(() => null)) as { password?: unknown } | null;
  const password = typeof payload?.password === "string" ? payload.password : "";
  const [providedToken, expectedToken] = await Promise.all([
    createDemoAccessToken(password),
    createDemoAccessToken(getDemoPassword()),
  ]);

  if (providedToken !== expectedToken) {
    return NextResponse.json({ error: "invalid_password" }, { status: 401 });
  }

  const response = NextResponse.json({ ok: true, profiles: DEMO_PROFILES });
  response.cookies.set(DEMO_ACCESS_COOKIE, expectedToken, {
    httpOnly: true,
    maxAge: COOKIE_MAX_AGE,
    path: "/",
    sameSite: "lax",
    secure: usesHttps(request),
  });
  response.cookies.set(DEMO_PROFILE_COOKIE, "", {
    httpOnly: true,
    maxAge: 0,
    path: "/",
    sameSite: "lax",
    secure: usesHttps(request),
  });
  return response;
}

export async function PUT(request: NextRequest) {
  const expectedToken = await createDemoAccessToken(getDemoPassword());
  if (request.cookies.get(DEMO_ACCESS_COOKIE)?.value !== expectedToken) {
    return NextResponse.json({ error: "demo_access_required" }, { status: 401 });
  }

  const payload = (await request.json().catch(() => null)) as { profileId?: unknown } | null;
  const profile = findDemoProfile(typeof payload?.profileId === "string" ? payload.profileId : undefined);
  if (!profile) return NextResponse.json({ error: "invalid_profile" }, { status: 400 });

  const response = NextResponse.json({ ok: true });
  response.cookies.set(DEMO_PROFILE_COOKIE, profile.id, {
    httpOnly: true,
    maxAge: COOKIE_MAX_AGE,
    path: "/",
    sameSite: "lax",
    secure: usesHttps(request),
  });
  return response;
}

export async function DELETE(request: NextRequest) {
  const response = NextResponse.json({ ok: true });
  response.cookies.set(DEMO_ACCESS_COOKIE, "", {
    httpOnly: true,
    maxAge: 0,
    path: "/",
    sameSite: "lax",
    secure: usesHttps(request),
  });
  response.cookies.set(DEMO_PROFILE_COOKIE, "", {
    httpOnly: true,
    maxAge: 0,
    path: "/",
    sameSite: "lax",
    secure: usesHttps(request),
  });
  return response;
}
