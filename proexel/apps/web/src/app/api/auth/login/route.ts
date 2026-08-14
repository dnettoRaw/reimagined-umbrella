import { NextResponse } from "next/server";

import { SESSION_COOKIE, signSession } from "@/lib/proexel/auth-token";
import type { Role } from "@/lib/proexel/types";

import { scryptSync, timingSafeEqual } from "node:crypto";

interface AuthUser {
  id: string;
  email: string;
  name: string;
  role: Role;
  password_hash: string;
}

const attempts = new Map<string, { count: number; resetAt: number }>();
const WINDOW_MS = 15 * 60 * 1000;
const MAX_ATTEMPTS = 5;

export async function POST(request: Request) {
  const client = request.headers.get("x-forwarded-for")?.split(",")[0]?.trim() || "local";
  const now = Date.now();
  const rate = attempts.get(client);
  if (rate && rate.resetAt > now && rate.count >= MAX_ATTEMPTS) {
    return NextResponse.json({ error: "Muitas tentativas. Tente novamente mais tarde." }, { status: 429 });
  }
  const body = (await request.json().catch(() => null)) as {
    email?: unknown;
    password?: unknown;
    remember?: unknown;
    next?: unknown;
  } | null;
  const email = typeof body?.email === "string" ? body.email.trim().toLowerCase() : "";
  const password = typeof body?.password === "string" ? body.password : "";
  const users = readUsers();
  const user = users.find((candidate) => candidate.email.toLowerCase() === email);
  if (!user || !verifyPassword(password, user.password_hash)) {
    const active = rate && rate.resetAt > now ? rate : { count: 0, resetAt: now + WINDOW_MS };
    attempts.set(client, { ...active, count: active.count + 1 });
    return NextResponse.json({ error: "Email ou palavra-passe inválidos." }, { status: 401 });
  }
  attempts.delete(client);
  const remember = body?.remember === true;
  const maxAge = remember ? 30 * 24 * 60 * 60 : 8 * 60 * 60;
  const secret = process.env.PROEXEL_SESSION_SECRET;
  if (!secret || secret.length < 32) {
    return NextResponse.json({ error: "Autenticação não configurada." }, { status: 503 });
  }
  const token = await signSession(
    { sub: user.id, email: user.email, name: user.name, role: user.role, exp: now + maxAge * 1000 },
    secret,
  );
  const requestedNext = typeof body?.next === "string" ? body.next : "";
  const next = requestedNext.startsWith("/dashboard") ? requestedNext : "/dashboard/overview";
  const response = NextResponse.json({ ok: true, next, user: { name: user.name, role: user.role } });
  response.cookies.set(SESSION_COOKIE, token, {
    httpOnly: true,
    sameSite: "strict",
    secure: process.env.NODE_ENV === "production",
    path: "/",
    maxAge,
  });
  return response;
}

function readUsers(): AuthUser[] {
  try {
    const value = JSON.parse(process.env.PROEXEL_AUTH_USERS ?? "[]") as AuthUser[];
    return Array.isArray(value) ? value : [];
  } catch {
    return [];
  }
}

function verifyPassword(password: string, encoded: string): boolean {
  const [algorithm, salt, expectedHex, extra] = encoded.split("$");
  if (algorithm !== "scrypt" || !salt || !expectedHex || extra || password.length < 8) return false;
  try {
    const expected = Buffer.from(expectedHex, "hex");
    const actual = scryptSync(password, salt, expected.length);
    return expected.length > 0 && timingSafeEqual(expected, actual);
  } catch {
    return false;
  }
}
