import type { IdentityRecord } from "./types";

const CAPABILITY = "proexel.identity.resolve";

export async function resolveIdentity(criteria: {
  email?: string;
  id?: string;
}): Promise<IdentityRecord | null | undefined> {
  const serviceUrl = process.env.PROEXEL_SERVICE_URL;
  const token = capabilityToken(CAPABILITY);
  if (!serviceUrl || !token) return undefined;
  try {
    const response = await fetch(`${serviceUrl}/v1/query`, {
      method: "POST",
      headers: { authorization: `Bearer ${token}`, "content-type": "application/json" },
      cache: "no-store",
      body: JSON.stringify({
        query_name: CAPABILITY,
        query_id: `identity-${crypto.randomUUID()}`,
        payload: { data: criteria },
      }),
    });
    if (!response.ok) return undefined;
    const body = (await response.json()) as { ok?: boolean; payload?: { user?: IdentityRecord | null } };
    return body.ok ? (body.payload?.user ?? null) : undefined;
  } catch {
    return undefined;
  }
}

function capabilityToken(capability: string): string | undefined {
  try {
    const tokens = JSON.parse(process.env.PROEXEL_SERVICE_TOKENS ?? "{}") as Record<string, string>;
    return tokens[capability] ?? process.env.PROEXEL_SERVICE_TOKEN;
  } catch {
    return process.env.PROEXEL_SERVICE_TOKEN;
  }
}
