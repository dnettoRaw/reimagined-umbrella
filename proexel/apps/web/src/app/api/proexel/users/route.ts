import { NextResponse } from "next/server";

import type { TranslationKey } from "@/lib/i18n/messages";
import { getI18n } from "@/lib/i18n/server";
import { getCurrentSession } from "@/lib/proexel/auth-server";
import { can } from "@/lib/proexel/permissions";
import { executeCommand, listUsers, ProexelServiceError } from "@/lib/proexel/service";
import type { Role } from "@/lib/proexel/types";

import { randomBytes, scryptSync } from "node:crypto";

const ROLES: Role[] = ["admin", "chefe", "compras", "tecnico"];

export async function GET() {
  const denied = await authorize();
  if (denied) return denied;
  return NextResponse.json(await listUsers());
}

export async function POST(request: Request) {
  const denied = await authorize();
  if (denied) return denied;
  const body = await readBody(request);
  const name = text(body.name);
  const email = text(body.email);
  const selectedRole = role(body.role);
  const password = text(body.password);
  const pin = text(body.pin);
  const maximumRepairLevel = repairLevel(body.maximum_repair_level);
  if (!name || !email || !selectedRole || !maximumRepairLevel) return validationError("service.invalidData");
  if (!validPassword(password) || (pin && !validPin(pin))) return validationError("service.credentialFormat");
  return run("proexel.admin.users.create", {
    email,
    name,
    role: selectedRole,
    password_hash: hashCredential(password),
    pin_hash: pin ? hashCredential(pin) : null,
    maximum_repair_level: maximumRepairLevel,
  });
}

export async function PATCH(request: Request) {
  const denied = await authorize();
  if (denied) return denied;
  const body = await readBody(request);
  const id = text(body.id);
  const name = text(body.name);
  const email = text(body.email);
  const selectedRole = role(body.role);
  const maximumRepairLevel = repairLevel(body.maximum_repair_level);
  if (!id || !name || !email || !selectedRole || !maximumRepairLevel || typeof body.active !== "boolean") {
    return validationError("service.invalidData");
  }
  return run("proexel.admin.users.update", {
    id,
    email,
    name,
    role: selectedRole,
    active: body.active,
    maximum_repair_level: maximumRepairLevel,
  });
}

export async function PUT(request: Request) {
  const denied = await authorize();
  if (denied) return denied;
  const body = await readBody(request);
  const password = text(body.password);
  const pin = text(body.pin);
  const clearPin = body.clear_pin === true;
  if (
    !text(body.id) ||
    (pin && clearPin) ||
    (!password && !pin && !clearPin) ||
    (password && !validPassword(password)) ||
    (pin && !validPin(pin))
  ) {
    return validationError("service.credentialFormat");
  }
  return run("proexel.admin.users.reset_credentials", {
    id: text(body.id),
    password_hash: password ? hashCredential(password) : null,
    pin_hash: pin ? hashCredential(pin) : null,
    clear_pin: clearPin,
  });
}

async function authorize() {
  const session = await getCurrentSession();
  if (!session) return NextResponse.json({ accepted: false, message: "unauthorized" }, { status: 401 });
  if (!can("admin.users.manage", session.role)) {
    return NextResponse.json({ accepted: false, message: "forbidden" }, { status: 403 });
  }
  return null;
}

async function readBody(request: Request) {
  return (await request.json().catch(() => ({}))) as Record<string, unknown>;
}

async function run(capability: string, data: Record<string, unknown>) {
  try {
    return NextResponse.json(await executeCommand(capability, data));
  } catch (error) {
    const failure = error instanceof ProexelServiceError ? error : new ProexelServiceError("invalid_data", 400);
    return NextResponse.json({ accepted: false, message: failure.message }, { status: failure.status });
  }
}

async function validationError(key: TranslationKey) {
  const { t } = await getI18n();
  return NextResponse.json({ accepted: false, message: t(key) }, { status: 400 });
}

function text(value: unknown) {
  return typeof value === "string" ? value.trim() : "";
}

function role(value: unknown): Role | "" {
  return ROLES.includes(value as Role) ? (value as Role) : "";
}

function repairLevel(value: unknown) {
  const level = typeof value === "number" ? value : Number(value);
  return Number.isInteger(level) && level >= 1 && level <= 5 ? level : 0;
}

function validPassword(value: string) {
  return value.length >= 8 && value.length <= 128;
}

function validPin(value: string) {
  return /^\d{4,8}$/.test(value);
}

function hashCredential(value: string) {
  const salt = randomBytes(16).toString("hex");
  return `scrypt$${salt}$${scryptSync(value, salt, 32).toString("hex")}`;
}
