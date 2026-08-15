import { NextResponse } from "next/server";

import { getCurrentSession } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";

import { randomUUID } from "node:crypto";
import { mkdir, readFile, unlink, writeFile } from "node:fs/promises";
import path from "node:path";

export const runtime = "nodejs";

const TYPES = {
  "image/png": { extension: "png", magic: [0x89, 0x50, 0x4e, 0x47] },
  "image/jpeg": { extension: "jpg", magic: [0xff, 0xd8, 0xff] },
  "image/webp": { extension: "webp", magic: [0x52, 0x49, 0x46, 0x46] },
} as const;

type AttachmentKind = "valve-photos" | "signatures";

export async function POST(request: Request) {
  const session = await getCurrentSession();
  if (!session) return NextResponse.json({ error: "unauthorized" }, { status: 401 });
  const form = await request.formData().catch(() => null);
  const file = form?.get("file");
  const kind = form?.get("kind");
  if (!(file instanceof File) || (kind !== "valve-photos" && kind !== "signatures")) {
    return NextResponse.json({ error: "invalid_attachment" }, { status: 400 });
  }
  const permission = kind === "valve-photos" ? "valve.update_photo" : "maintenance.register";
  if (!can(permission, session.role)) return NextResponse.json({ error: "forbidden" }, { status: 403 });
  const spec = TYPES[file.type as keyof typeof TYPES];
  const maxBytes = kind === "valve-photos" ? 5 * 1024 * 1024 : 1024 * 1024;
  if (!spec || file.size === 0 || file.size > maxBytes) {
    return NextResponse.json({ error: "invalid_attachment" }, { status: 400 });
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  if (!matchesMagic(bytes, file.type, spec.magic)) {
    return NextResponse.json({ error: "invalid_attachment" }, { status: 400 });
  }
  const ref = `${kind}/${randomUUID()}.${spec.extension}`;
  const destination = resolveAttachment(ref);
  await mkdir(path.dirname(destination), { recursive: true, mode: 0o700 });
  await writeFile(destination, bytes, { mode: 0o600, flag: "wx" });
  return NextResponse.json({ ref });
}

export async function GET(request: Request) {
  const session = await getCurrentSession();
  if (!session) return NextResponse.json({ error: "unauthorized" }, { status: 401 });
  const ref = new URL(request.url).searchParams.get("ref") ?? "";
  const kind = attachmentKind(ref);
  if (!kind) return NextResponse.json({ error: "invalid_attachment" }, { status: 400 });
  const permission = kind === "valve-photos" ? "valve.read" : "maintenance.read";
  if (!can(permission, session.role)) return NextResponse.json({ error: "forbidden" }, { status: 403 });
  try {
    const bytes = await readFile(/* turbopackIgnore: true */ resolveAttachment(ref));
    return new NextResponse(bytes, {
      headers: {
        "cache-control": "private, max-age=300",
        "content-type": contentType(ref),
        "x-content-type-options": "nosniff",
      },
    });
  } catch {
    return NextResponse.json({ error: "not_found" }, { status: 404 });
  }
}

export async function DELETE(request: Request) {
  const session = await getCurrentSession();
  if (!session) return NextResponse.json({ error: "unauthorized" }, { status: 401 });
  const body = (await request.json().catch(() => null)) as { ref?: unknown } | null;
  const ref = typeof body?.ref === "string" ? body.ref : "";
  const kind = attachmentKind(ref);
  if (!kind) return NextResponse.json({ error: "invalid_attachment" }, { status: 400 });
  const permission = kind === "valve-photos" ? "valve.update_photo" : "maintenance.register";
  if (!can(permission, session.role)) return NextResponse.json({ error: "forbidden" }, { status: 403 });
  await unlink(resolveAttachment(ref)).catch(() => undefined);
  return NextResponse.json({ deleted: true });
}

function attachmentRoot() {
  return path.resolve(
    /* turbopackIgnore: true */
    process.env.PROEXEL_ATTACHMENTS_DIR ?? path.join(process.cwd(), "../service/target/runtime/attachments"),
  );
}

function resolveAttachment(ref: string) {
  const root = attachmentRoot();
  const target = path.resolve(root, ref);
  if (!target.startsWith(`${root}${path.sep}`)) throw new Error("invalid_attachment_path");
  return target;
}

function attachmentKind(ref: string): AttachmentKind | null {
  const match = /^(valve-photos|signatures)\/[0-9a-f-]+\.(png|jpg|webp)$/.exec(ref);
  return match ? (match[1] as AttachmentKind) : null;
}

function matchesMagic(bytes: Uint8Array, type: string, magic: readonly number[]) {
  if (!magic.every((value, index) => bytes[index] === value)) return false;
  return type !== "image/webp" || String.fromCharCode(...bytes.slice(8, 12)) === "WEBP";
}

function contentType(ref: string) {
  if (ref.endsWith(".png")) return "image/png";
  if (ref.endsWith(".webp")) return "image/webp";
  return "image/jpeg";
}
