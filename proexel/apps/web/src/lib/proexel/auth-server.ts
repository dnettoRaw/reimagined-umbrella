import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { type ProexelSession, SESSION_COOKIE, verifySession } from "./auth-token";
import { resolveIdentity } from "./identity-service";
import { can } from "./permissions";

export async function getCurrentSession(): Promise<ProexelSession | null> {
  const token = (await cookies()).get(SESSION_COOKIE)?.value;
  const session = await verifySession(token, process.env.PROEXEL_SESSION_SECRET);
  if (!session) return null;
  const identity = await resolveIdentity({ id: session.sub });
  if (identity === undefined) return session;
  if (!identity?.active || identity.auth_version !== session.ver || identity.role !== session.role) return null;
  return session;
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
