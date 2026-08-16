import { cookies } from "next/headers";

import {
  applyDemoOperations,
  createDemoState,
  DEMO_OPERATIONS_COOKIE,
  type DemoState,
  decodeDemoOperations,
} from "./demo-data";
import type {
  AuditListResult,
  ItemCategory,
  ListResult,
  MachineListResult,
  OperatorSummary,
  OverviewResult,
  ReportResult,
  ServiceOrderStatus,
} from "./types";

export function isDemoMode() {
  return process.env.PROEXEL_DEMO === "1" || process.env.NEXT_PUBLIC_PROEXEL_DEMO === "1";
}

export async function getDemoState(): Promise<DemoState> {
  const encoded = (await cookies()).get(DEMO_OPERATIONS_COOKIE)?.value;
  return applyDemoOperations(createDemoState(), decodeDemoOperations(encoded));
}

const result = <T>(items: T[]): ListResult<T> => ({ items, total: items.length, schema_version: 2, source: "appcore" });

export async function getDemoOverview(): Promise<OverviewResult> {
  return overview(await getDemoState());
}

export async function listDemoCategories(payload: Record<string, unknown>): Promise<ListResult<ItemCategory>> {
  const state = await getDemoState();
  const search = String(payload.search ?? "").toLowerCase();
  return result(
    state.categories.filter((item) => !search || `${item.code} ${item.name}`.toLowerCase().includes(search)),
  );
}

export async function listDemoMachines(payload: Record<string, unknown>): Promise<MachineListResult> {
  const state = await getDemoState();
  const id = String(payload.id ?? "");
  const search = String(payload.search ?? "").toLowerCase();
  const zone = String(payload.zone ?? "");
  const status = String(payload.status ?? "");
  const page = Math.max(1, Number(payload.page ?? 1));
  const pageSize = Math.max(1, Number(payload.page_size ?? 25));
  const all = state.machines.filter(
    (item) =>
      (!id || item.id === id) &&
      (!search || `${item.code} ${item.name} ${item.zone}`.toLowerCase().includes(search)) &&
      (!zone || item.zone === zone) &&
      (!status || item.status === status),
  );
  return {
    ...result(all.slice((page - 1) * pageSize, page * pageSize)),
    total: all.length,
    page,
    page_size: pageSize,
    facets: { zones: [...new Set(state.machines.map((item) => item.zone))].sort() },
  };
}

export async function listDemoOrders(payload: Record<string, unknown>) {
  const state = await getDemoState();
  const id = String(payload.id ?? "");
  const machineId = String(payload.machine_id ?? "");
  const status = String(payload.status ?? "");
  const operatorId = String(payload.operator_id ?? "");
  return result(
    state.orders.filter(
      (order) =>
        (!id || order.id === id) &&
        (!machineId || order.machine_id === machineId) &&
        (!status || order.status === status) &&
        (!operatorId || order.tasks.some((task) => task.assigned_operator_id === operatorId)),
    ),
  );
}

export async function listDemoInspections(payload: Record<string, unknown>) {
  const state = await getDemoState();
  return result(
    state.inspections.filter((inspection) => {
      const filters: Array<[string, unknown]> = [
        ["id", inspection.id],
        ["service_order_id", inspection.service_order_id],
        ["machine_id", inspection.machine_id],
        ["machine_item_id", inspection.machine_item_id],
        ["operator_id", inspection.operator_id],
      ];
      return filters.every(([key, value]) => !payload[key] || payload[key] === value);
    }),
  );
}

export async function listDemoStock() {
  return result((await getDemoState()).stock);
}
export async function listDemoRestockRequests() {
  return result((await getDemoState()).restockRequests);
}
export async function listDemoSuppliers() {
  return result((await getDemoState()).suppliers);
}
export async function listDemoUsers() {
  return result((await getDemoState()).users);
}

export async function listDemoOperators(): Promise<ListResult<OperatorSummary>> {
  const users = (await getDemoState()).users.filter((user) => user.role !== "compras");
  return result(
    users.map(({ id, name, role, active, maximum_repair_level }) => ({
      id,
      name,
      role,
      active,
      maximum_repair_level,
    })) as OperatorSummary[],
  );
}

export async function listDemoAudit(payload: Record<string, unknown>): Promise<AuditListResult> {
  const state = await getDemoState();
  const query = String(payload.search ?? payload.q ?? "").toLowerCase();
  const actor = String(payload.actor ?? "");
  const operation = String(payload.operation ?? "");
  const aggregate = String(payload.aggregate ?? "");
  const items = state.audit.filter(
    (event) =>
      (!query || `${event.actor} ${event.description} ${event.aggregate_id}`.toLowerCase().includes(query)) &&
      (!actor || event.actor === actor) &&
      (!operation || event.operation === operation) &&
      (!aggregate || event.aggregate === aggregate),
  );
  return {
    ...result(items),
    total: items.length,
    page: 1,
    page_size: 50,
    operations: [...new Set(state.audit.map((event) => event.operation))].sort(),
    actors: [...new Set(state.audit.map((event) => event.actor))].sort(),
    aggregates: [...new Set(state.audit.map((event) => event.aggregate))].sort(),
  };
}

export async function getDemoReports(): Promise<ReportResult> {
  const state = await getDemoState();
  const reportOverview = overview(state);
  const zones = [...new Set(state.machines.map((machine) => machine.zone))];
  return {
    schema_version: 2,
    source: "appcore",
    generated_at_ms: Date.now(),
    overview: reportOverview,
    by_zone: zones.map((zone) => {
      const machines = state.machines.filter((machine) => machine.zone === zone);
      const items = machines.flatMap((machine) => machine.items);
      return {
        zone,
        machines: machines.length,
        items: items.length,
        critical_items: items.filter((item) => item.status === "critical" || item.status === "maintenance_required")
          .length,
      };
    }),
    critical_items: state.machines.flatMap((machine) =>
      machine.items
        .filter((item) => item.status === "critical" || item.status === "maintenance_required")
        .map((item) => ({ item, machine, category: item.category ?? null })),
    ),
    recent_inspections: state.inspections.toSorted(
      (left, right) => (right.completed_at_ms ?? right.started_at_ms) - (left.completed_at_ms ?? left.started_at_ms),
    ),
  };
}

function overview(state: DemoState): OverviewResult {
  const items = state.machines.flatMap((machine) => machine.items.filter((item) => item.active));
  const byStatus = <T extends { status: string }>(entries: T[]) =>
    Object.fromEntries(
      [...new Set(entries.map((entry) => entry.status))].map((status) => [
        status,
        entries.filter((entry) => entry.status === status).length,
      ]),
    );
  const orderStatuses: Record<ServiceOrderStatus, number> = { pending: 0, in_progress: 0, completed: 0, cancelled: 0 };
  for (const order of state.orders) orderStatuses[order.status] += 1;
  return {
    schema_version: 2,
    source: "appcore",
    machines: { total: state.machines.length, by_status: byStatus(state.machines) },
    machine_items: { total: items.length, by_status: byStatus(items) },
    orders: orderStatuses,
    stock: {
      total: state.stock.length,
      low: state.stock.filter((item) => item.quantity <= item.minimum_quantity).length,
    },
    recent_inspections: state.inspections.slice(0, 5),
    upcoming_orders: state.orders.filter((order) => order.status === "pending").slice(0, 5),
  };
}
