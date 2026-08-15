import type { TranslationKey } from "../i18n/messages";
import { getI18n } from "../i18n/server";
import { getCurrentSession } from "./auth-server";
import { can } from "./permissions";
import type {
  AuditEvent,
  CommandResult,
  ListResult,
  MaintenanceRecord,
  OverviewResult,
  ReportResult,
  RestockRequest,
  ServiceOrder,
  StockItem,
  Supplier,
  Valve,
  ValveListResult,
} from "./types";

const COMMAND_PERMISSIONS: Record<string, string> = {
  "proexel.valves.create": "valve.create",
  "proexel.valves.update": "valve.update_technical_fields",
  "proexel.maintenance.register": "maintenance.register",
  "proexel.orders.create": "order.create",
  "proexel.orders.change_status": "order.change_status",
  "proexel.purchasing.create_restock_request": "restock.create_suggestion",
  "proexel.purchasing.review_restock_request": "restock.approve_reject",
  "proexel.stock.adjust": "stock.adjust_quantity",
  "proexel.stock.upsert_item": "stock.add_or_increment",
  "proexel.suppliers.create": "supplier.create_update_delete",
  "proexel.suppliers.update": "supplier.create_update_delete",
};

const QUERY_PERMISSIONS: Record<string, string> = {
  "proexel.valves.list": "valve.read",
  "proexel.maintenance.list": "maintenance.read",
  "proexel.orders.list": "order.read",
  "proexel.purchasing.list_restock_requests": "restock.read",
  "proexel.stock.list": "stock.read",
  "proexel.suppliers.list": "supplier.read",
  "proexel.audit.list": "audit.read",
  "proexel.reports.get": "report.read",
};

const SERVICE_ERROR_KEYS: Record<string, TranslationKey> = {
  invalid_json_payload: "service.invalidPayload",
  invalid_command_data: "service.invalidPayload",
  unknown_command: "service.unknownCommand",
  forbidden: "service.permissionDenied",
  tag_already_exists: "service.tagExists",
  valve_not_found: "service.valveNotFound",
  order_not_found: "service.orderNotFound",
  invalid_order_status_transition: "service.invalidOrderTransition",
  review_status_must_be_final: "service.reviewMustBeFinal",
  restock_request_not_found: "service.restockNotFound",
  restock_request_already_reviewed: "service.restockReviewed",
  adjustment_delta_cannot_be_zero: "service.zeroAdjustment",
  stock_item_not_found: "service.stockNotFound",
  stock_cannot_be_negative: "service.stockNegative",
  stock_quantity_overflow: "service.stockOverflow",
  supplier_not_found: "service.supplierNotFound",
};

const EMPTY_OVERVIEW: OverviewResult = {
  schema_version: 1,
  source: "unavailable",
  valves: { total: 0, ok: 0, warning: 0, critical: 0 },
  orders: { open: 0, in_progress: 0, completed: 0 },
  stock: { low: 0, total: 0 },
  recent_maintenance: [],
  upcoming_orders: [],
};

export class ProexelServiceError extends Error {
  constructor(
    message: string,
    readonly status = 503,
  ) {
    super(message);
  }
}

function capabilityToken(capability: string): string | undefined {
  const raw = process.env.PROEXEL_SERVICE_TOKENS;
  if (raw) {
    try {
      const tokens = JSON.parse(raw) as Record<string, string>;
      if (tokens[capability]) return tokens[capability];
    } catch {
      // A malformed map behaves like a missing scoped token.
    }
  }
  return process.env.PROEXEL_SERVICE_TOKEN;
}

async function query<T>(capability: string, fallback: T, payload: Record<string, unknown> = {}): Promise<T> {
  const session = await getCurrentSession();
  if (!session) return fallback;
  const permission = QUERY_PERMISSIONS[capability];
  if (permission && !can(permission, session.role)) return fallback;
  const serviceUrl = process.env.PROEXEL_SERVICE_URL;
  const token = capabilityToken(capability);
  if (!serviceUrl || !token) return fallback;
  try {
    const response = await fetch(`${serviceUrl}/v1/query`, {
      method: "POST",
      headers: { authorization: `Bearer ${token}`, "content-type": "application/json" },
      cache: "no-store",
      body: JSON.stringify({ query_name: capability, query_id: `web-${crypto.randomUUID()}`, payload }),
    });
    if (!response.ok) return fallback;
    const body = (await response.json()) as { ok?: boolean; payload?: T };
    return body.ok && body.payload ? body.payload : fallback;
  } catch {
    return fallback;
  }
}

function emptyList<T>(): ListResult<T> {
  return { items: [], schema_version: 1, source: "unavailable" };
}

export async function executeCommand(capability: string, data: Record<string, unknown>): Promise<CommandResult> {
  const { t } = await getI18n();
  const session = await getCurrentSession();
  if (!session) throw new ProexelServiceError(t("service.invalidSession"), 401);
  const serviceUrl = process.env.PROEXEL_SERVICE_URL;
  const token = capabilityToken(capability);
  if (!serviceUrl) throw new ProexelServiceError(t("service.notConfigured"));
  if (!token) throw new ProexelServiceError(t("service.missingToken", { capability }), 401);
  const commandId = `web-${crypto.randomUUID()}`;
  const permission = COMMAND_PERMISSIONS[capability];
  if (!permission || !can(permission, session.role)) {
    throw new ProexelServiceError(t("service.forbidden", { role: t(`role.${session.role}`), capability }), 403);
  }
  const response = await fetch(`${serviceUrl}/v1/command`, {
    method: "POST",
    headers: { authorization: `Bearer ${token}`, "content-type": "application/json" },
    body: JSON.stringify({
      command_name: capability,
      command_id: commandId,
      idempotency_key: commandId,
      payload: JSON.stringify({
        actor: {
          id: session.sub,
          name: session.name,
          role: session.role,
        },
        data,
      }),
    }),
  });
  const body = (await response.json().catch(() => ({}))) as CommandResult;
  if (!response.ok || !body.accepted) {
    const messageKey = body.message ? SERVICE_ERROR_KEYS[body.message.split(":", 1)[0]] : undefined;
    throw new ProexelServiceError(messageKey ? t(messageKey) : t("command.rejected"), response.status || 400);
  }
  return body;
}

export async function getOverview(): Promise<OverviewResult> {
  const result = await query("proexel.overview.get", EMPTY_OVERVIEW);
  return { ...result, source: result === EMPTY_OVERVIEW ? "unavailable" : "appcore" };
}

export async function listValves(payload: Record<string, unknown> = {}): Promise<ValveListResult> {
  const fallback = emptyList<Valve>();
  const result = await query("proexel.valves.list", fallback, payload);
  return { ...result, source: result === fallback ? "unavailable" : "appcore" };
}

async function listQuery<T>(capability: string): Promise<ListResult<T>> {
  const fallback = emptyList<T>();
  const result = await query(capability, fallback);
  return { ...result, source: result === fallback ? "unavailable" : "appcore" };
}

export const listMaintenance = () => listQuery<MaintenanceRecord>("proexel.maintenance.list");
export const listServiceOrders = () => listQuery<ServiceOrder>("proexel.orders.list");
export const listRestockRequests = () => listQuery<RestockRequest>("proexel.purchasing.list_restock_requests");
export const listStock = () => listQuery<StockItem>("proexel.stock.list");
export const listSuppliers = () => listQuery<Supplier>("proexel.suppliers.list");
export const listAudit = () => listQuery<AuditEvent>("proexel.audit.list");

export async function getReports(): Promise<ReportResult> {
  const fallback: ReportResult = {
    schema_version: 1,
    source: "unavailable",
    generated_at_ms: 0,
    overview: EMPTY_OVERVIEW,
    by_zone: [],
    critical_valves: [],
    recent_maintenance: [],
  };
  const result = await query("proexel.reports.get", fallback);
  return { ...result, source: result === fallback ? "unavailable" : "appcore" };
}

export async function getRuntimeStatus() {
  const url = process.env.PROEXEL_SERVICE_URL;
  if (!url) return { configured: false, healthy: false, url: null as string | null };
  try {
    const response = await fetch(`${url}/v1/health`, { cache: "no-store" });
    return { configured: true, healthy: response.ok, url };
  } catch {
    return { configured: true, healthy: false, url };
  }
}
