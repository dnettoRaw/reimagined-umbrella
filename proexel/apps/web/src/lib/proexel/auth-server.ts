import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { type ProexelSession, SESSION_COOKIE, verifySession } from "./auth-token";
import { can } from "./permissions";

export async function getCurrentSession(): Promise<ProexelSession | null> {
  const token = (await cookies()).get(SESSION_COOKIE)?.value;
  return verifySession(token, process.env.PROEXEL_SESSION_SECRET);
}

export async function requireSession(): Promise<ProexelSession> {
  const session = await getCurrentSession();
  if (!session) redirect("/auth/login");
  return session;
}

export async function requirePermission(permission: string): Promise<ProexelSession> {
  const session = await requireSession();
  if (!can(permission, session.role)) redirect("/unauthorized");
  return session;
}
