import type { TranslationKey } from "../i18n/messages";
import { getI18n } from "../i18n/server";
import { getCurrentSession } from "./auth-server";
import {
  getDemoOverview,
  getDemoReports,
  isDemoMode,
  listDemoAudit,
  listDemoCategories,
  listDemoInspections,
  listDemoMachines,
  listDemoOperators,
  listDemoOrders,
  listDemoRestockRequests,
  listDemoStock,
  listDemoSuppliers,
  listDemoUsers,
} from "./demo-service";
import { can } from "./permissions";
import type {
  AuditEvent,
  AuditListResult,
  CommandResult,
  ItemCategory,
  ItemInspection,
  ListResult,
  Machine,
  MachineListResult,
  OperatorSummary,
  OverviewResult,
  ReportResult,
  RestockRequest,
  ServiceOrder,
  StockItem,
  Supplier,
  UserAccount,
} from "./types";

const COMMAND_PERMISSIONS: Record<string, string> = {
  "proexel.item_categories.create": "item_category.manage",
  "proexel.item_categories.update": "item_category.manage",
  "proexel.machines.create": "machine.create",
  "proexel.machines.update": "machine.update",
  "proexel.machine_items.add": "machine_item.manage",
  "proexel.machine_items.update": "machine_item.manage",
  "proexel.machine_items.reorder": "machine_item.manage",
  "proexel.machine_items.remove": "machine_item.manage",
  "proexel.machine_items.replace": "machine_item.manage",
  "proexel.photos.add": "photo.manage_reference",
  "proexel.photos.delete": "photo.manage_reference",
  "proexel.orders.create": "order.create",
  "proexel.orders.start": "inspection.execute",
  "proexel.orders.assign_task": "order.manage",
  "proexel.orders.delete": "order.delete",
  "proexel.orders.complete": "inspection.execute",
  "proexel.inspections.start": "inspection.execute",
  "proexel.inspections.complete": "inspection.execute",
  "proexel.purchasing.create_restock_request": "restock.create_suggestion",
  "proexel.purchasing.review_restock_request": "restock.approve_reject",
  "proexel.purchasing.delete_restock_request": "restock.delete",
  "proexel.stock.adjust": "stock.adjust_quantity",
  "proexel.stock.upsert_item": "stock.add_or_increment",
  "proexel.stock.delete_item": "stock.delete",
  "proexel.suppliers.create": "supplier.create_update_delete",
  "proexel.suppliers.update": "supplier.create_update_delete",
  "proexel.suppliers.delete": "supplier.create_update_delete",
  "proexel.admin.users.create": "admin.users.manage",
  "proexel.admin.users.update": "admin.users.manage",
  "proexel.admin.users.reset_credentials": "admin.users.manage",
};

const QUERY_PERMISSIONS: Record<string, string> = {
  "proexel.item_categories.list": "item_category.read",
  "proexel.machines.list": "machine.read",
  "proexel.orders.list": "order.read",
  "proexel.inspections.list": "inspection.read",
  "proexel.purchasing.list_restock_requests": "restock.read",
  "proexel.stock.list": "stock.read",
  "proexel.suppliers.list": "supplier.read",
  "proexel.audit.list": "audit.read",
  "proexel.reports.get": "report.read",
  "proexel.admin.users.list": "admin.users.manage",
  "proexel.operators.list": "operator.read",
};

const SERVICE_ERROR_KEYS: Record<string, TranslationKey> = {
  invalid_json_payload: "service.invalidPayload",
  invalid_command_data: "service.invalidPayload",
  unknown_command: "service.unknownCommand",
  forbidden: "service.permissionDenied",
  category_not_found: "service.categoryNotFound",
  category_inactive: "service.categoryInactive",
  category_code_already_exists: "service.categoryCodeExists",
  machine_not_found: "service.machineNotFound",
  machine_code_already_exists: "service.machineCodeExists",
  machine_item_not_found: "service.machineItemNotFound",
  machine_item_code_already_exists: "service.machineItemCodeExists",
  operator_repair_level_insufficient: "service.repairLevelInsufficient",
  order_not_found: "service.orderNotFound",
  order_tasks_pending: "service.orderTasksPending",
  service_order_has_pending_tasks: "service.orderTasksPending",
  inspection_not_found: "service.inspectionNotFound",
  inspection_required_step_missing: "service.requiredStepMissing",
  inspection_photo_required: "service.photoRequired",
  inspection_measurement_unit_invalid: "service.measurementUnitInvalid",
  review_status_must_be_final: "service.reviewMustBeFinal",
  restock_request_not_found: "service.restockNotFound",
  restock_request_already_reviewed: "service.restockReviewed",
  adjustment_delta_cannot_be_zero: "service.zeroAdjustment",
  stock_item_not_found: "service.stockNotFound",
  stock_cannot_be_negative: "service.stockNegative",
  stock_quantity_overflow: "service.stockOverflow",
  supplier_not_found: "service.supplierNotFound",
  approved_restock_cannot_be_deleted: "service.approvedRestockDelete",
  stock_item_not_empty: "service.stockNotEmpty",
  photo_not_found: "service.photoNotFound",
  photo_already_exists: "service.photoExists",
  photo_in_use_by_service_order: "service.photoInUse",
  supplier_email_invalid: "service.supplierEmailInvalid",
  supplier_website_invalid: "service.supplierWebsiteInvalid",
  user_not_found: "service.userNotFound",
  user_email_invalid: "service.userEmailInvalid",
  user_email_already_exists: "service.userEmailExists",
  password_hash_invalid: "service.passwordInvalid",
  pin_hash_invalid: "service.pinInvalid",
  credential_change_required: "service.credentialRequired",
  last_active_admin_required: "service.lastAdminRequired",
};

const EMPTY_OVERVIEW: OverviewResult = {
  schema_version: 2,
  source: "unavailable",
  machines: { total: 0, by_status: {} },
  machine_items: { total: 0, by_status: {} },
  orders: { pending: 0, in_progress: 0, completed: 0, cancelled: 0 },
  stock: { low: 0, total: 0 },
  recent_inspections: [],
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
      body: JSON.stringify({
        query_name: capability,
        query_id: `web-${crypto.randomUUID()}`,
        payload: {
          actor: { id: session.sub, name: session.name, role: session.role },
          data: payload,
        },
      }),
    });
    if (!response.ok) return fallback;
    const body = (await response.json()) as { ok?: boolean; payload?: T };
    return body.ok && body.payload ? body.payload : fallback;
  } catch {
    return fallback;
  }
}

function emptyList<T>(): ListResult<T> {
  return { items: [], schema_version: 2, source: "unavailable", total: 0 };
}

export async function executeCommand(capability: string, data: Record<string, unknown>): Promise<CommandResult> {
  if (isDemoMode()) {
    const commandId = crypto.randomUUID();
    const prefix = RESOURCE_PREFIXES[capability];
    return { accepted: true, message: "demo", resource_id: prefix ? `${prefix}demo-${commandId}` : undefined };
  }
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
        actor: { id: session.sub, name: session.name, role: session.role },
        data,
      }),
    }),
  });
  const body = (await response.json().catch(() => ({}))) as CommandResult;
  if (!response.ok || !body.accepted) {
    const code = body.message?.split(":", 1)[0];
    const messageKey = code ? SERVICE_ERROR_KEYS[code] : undefined;
    throw new ProexelServiceError(messageKey ? t(messageKey) : t("command.rejected"), response.status || 400);
  }
  const prefix = RESOURCE_PREFIXES[capability];
  return prefix ? { ...body, resource_id: `${prefix}${commandId}` } : body;
}

const RESOURCE_PREFIXES: Record<string, string> = {
  "proexel.item_categories.create": "category-",
  "proexel.machines.create": "machine-",
  "proexel.machine_items.add": "machine-item-",
  "proexel.photos.add": "photo-",
  "proexel.orders.create": "order-",
  "proexel.inspections.start": "inspection-",
};

export async function getOverview(): Promise<OverviewResult> {
  if (isDemoMode()) return getDemoOverview();
  const result = await query("proexel.overview.get", EMPTY_OVERVIEW);
  return { ...result, source: result === EMPTY_OVERVIEW ? "unavailable" : "appcore" };
}

export async function listItemCategories(payload: Record<string, unknown> = {}): Promise<ListResult<ItemCategory>> {
  if (isDemoMode()) return listDemoCategories(payload);
  return listQuery<ItemCategory>("proexel.item_categories.list", payload);
}

export async function listMachines(payload: Record<string, unknown> = {}): Promise<MachineListResult> {
  if (isDemoMode()) return listDemoMachines(payload);
  const fallback: MachineListResult = {
    ...emptyList<Machine>(),
    total: 0,
    page: 1,
    page_size: 25,
    facets: { zones: [] },
  };
  const result = await query("proexel.machines.list", fallback, payload);
  return { ...result, source: result === fallback ? "unavailable" : "appcore" };
}

async function listQuery<T>(capability: string, payload: Record<string, unknown> = {}): Promise<ListResult<T>> {
  const fallback = emptyList<T>();
  const result = await query(capability, fallback, payload);
  return { ...result, source: result === fallback ? "unavailable" : "appcore" };
}

export function listServiceOrders(payload: Record<string, unknown> = {}) {
  return isDemoMode() ? listDemoOrders(payload) : listQuery<ServiceOrder>("proexel.orders.list", payload);
}
export function listInspections(payload: Record<string, unknown> = {}) {
  return isDemoMode() ? listDemoInspections(payload) : listQuery<ItemInspection>("proexel.inspections.list", payload);
}
export function listRestockRequests() {
  return isDemoMode()
    ? listDemoRestockRequests()
    : listQuery<RestockRequest>("proexel.purchasing.list_restock_requests");
}
export function listStock() {
  return isDemoMode() ? listDemoStock() : listQuery<StockItem>("proexel.stock.list");
}
export function listSuppliers() {
  return isDemoMode() ? listDemoSuppliers() : listQuery<Supplier>("proexel.suppliers.list");
}

export async function listAudit(payload: Record<string, unknown> = {}): Promise<AuditListResult> {
  if (isDemoMode()) return listDemoAudit(payload);
  const fallback: AuditListResult = {
    ...emptyList<AuditEvent>(),
    total: 0,
    page: 1,
    page_size: 50,
    operations: [],
    actors: [],
    aggregates: [],
  };
  const result = await query("proexel.audit.list", fallback, payload);
  return { ...result, source: result === fallback ? "unavailable" : "appcore" };
}

export async function listUsers(): Promise<ListResult<UserAccount>> {
  if (isDemoMode()) return listDemoUsers();
  const fallback = emptyList<UserAccount>();
  const result = await query("proexel.admin.users.list", fallback);
  return { ...result, source: result === fallback ? "unavailable" : "appcore" };
}

export async function listOperators(): Promise<ListResult<OperatorSummary>> {
  if (isDemoMode()) return listDemoOperators();
  return listQuery<OperatorSummary>("proexel.operators.list");
}

export async function getReports(): Promise<ReportResult> {
  if (isDemoMode()) return getDemoReports();
  const fallback: ReportResult = {
    schema_version: 2,
    source: "unavailable",
    generated_at_ms: 0,
    overview: EMPTY_OVERVIEW,
    by_zone: [],
    critical_items: [],
    recent_inspections: [],
  };
  const result = await query("proexel.reports.get", fallback);
  return { ...result, source: result === fallback ? "unavailable" : "appcore" };
}

export async function getRuntimeStatus() {
  if (isDemoMode()) return { configured: true, healthy: true, url: "localStorage://proexel-demo" };
  const url = process.env.PROEXEL_SERVICE_URL;
  if (!url) return { configured: false, healthy: false, url: null as string | null };
  try {
    const response = await fetch(`${url}/v1/health`, { cache: "no-store" });
    return { configured: true, healthy: response.ok, url };
  } catch {
    return { configured: true, healthy: false, url };
  }
}
