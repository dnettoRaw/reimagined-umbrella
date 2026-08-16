import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { type ProexelSession, SESSION_COOKIE, verifySession } from "./auth-token";
import { DEMO_PROFILE_COOKIE, findDemoProfile } from "./demo-access";
import { isDemoMode } from "./demo-service";
import { resolveIdentity } from "./identity-service";
import { can } from "./permissions";

export async function getCurrentSession(): Promise<ProexelSession | null> {
  const cookieStore = await cookies();
  if (isDemoMode()) {
    const profile = findDemoProfile(cookieStore.get(DEMO_PROFILE_COOKIE)?.value);
    if (!profile) return null;
    return {
      sub: profile.id,
      email: profile.email,
      name: profile.name,
      role: profile.role,
      maximum_repair_level: profile.maximumRepairLevel,
      ver: 1,
      exp: Date.UTC(2099, 0, 1),
    };
  }
  const token = cookieStore.get(SESSION_COOKIE)?.value;
  const session = await verifySession(token, process.env.PROEXEL_SESSION_SECRET);
  if (!session) return null;
  const identity = await resolveIdentity({ id: session.sub });
  if (identity === undefined) return session;
  if (
    !identity?.active ||
    identity.auth_version !== session.ver ||
    identity.role !== session.role ||
    identity.maximum_repair_level !== session.maximum_repair_level
  )
    return null;
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
