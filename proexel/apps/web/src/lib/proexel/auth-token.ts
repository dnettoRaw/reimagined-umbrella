import type { ComplexityLevel, Role } from "./types";

export const SESSION_COOKIE = "proexel_session";

export interface ProexelSession {
  sub: string;
  email: string;
  name: string;
  role: Role;
  maximum_repair_level: ComplexityLevel;
  ver: number;
  exp: number;
}

const ROLES: Role[] = ["admin", "chefe", "compras", "tecnico"];

function encodeBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replaceAll("+", "-").replaceAll("/", "_").replace(/=+$/, "");
}

function decodeBase64Url(value: string): ArrayBuffer {
  const base64 = value
    .replaceAll("-", "+")
    .replaceAll("_", "/")
    .padEnd(Math.ceil(value.length / 4) * 4, "=");
  const binary = atob(base64);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0)).buffer;
}

async function hmacKey(secret: string) {
  return crypto.subtle.importKey("raw", new TextEncoder().encode(secret), { name: "HMAC", hash: "SHA-256" }, false, [
    "sign",
    "verify",
  ]);
}

export async function signSession(session: ProexelSession, secret: string): Promise<string> {
  const payload = encodeBase64Url(new TextEncoder().encode(JSON.stringify(session)));
  const signature = await crypto.subtle.sign("HMAC", await hmacKey(secret), new TextEncoder().encode(payload));
  return `${payload}.${encodeBase64Url(new Uint8Array(signature))}`;
}

export async function verifySession(
  token: string | undefined,
  secret: string | undefined,
): Promise<ProexelSession | null> {
  if (!token || !secret || secret.length < 32) return null;
  const [payload, encodedSignature, extra] = token.split(".");
  if (!payload || !encodedSignature || extra) return null;
  try {
    const valid = await crypto.subtle.verify(
      "HMAC",
      await hmacKey(secret),
      decodeBase64Url(encodedSignature),
      new TextEncoder().encode(payload),
    );
    if (!valid) return null;
    const session = JSON.parse(new TextDecoder().decode(decodeBase64Url(payload))) as ProexelSession;
    if (
      !session.sub ||
      !session.email ||
      !session.name ||
      !ROLES.includes(session.role) ||
      ![1, 2, 3, 4, 5].includes(session.maximum_repair_level) ||
      !Number.isSafeInteger(session.ver) ||
      !Number.isSafeInteger(session.exp) ||
      session.exp <= Date.now()
    ) {
      return null;
    }
    return session;
  } catch {
    return null;
  }
}
