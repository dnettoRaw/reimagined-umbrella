export type MaintenanceHealth = "ok" | "warning" | "critical";
export type Role = "admin" | "chefe" | "compras" | "tecnico";

export interface Valve {
  id: string;
  tag: string;
  tag_normalized: string;
  zone: string;
  manufacturer?: string | null;
  serial?: string | null;
  valve_type?: string | null;
  dn?: string | null;
  seat?: string | null;
  actuator?: string | null;
  manufactured_at?: string | null;
  kit_reference?: string | null;
  last_kit_changed_at?: string | null;
  last_maintenance_at?: string | null;
  health: MaintenanceHealth;
  photos: ValvePhoto[];
}

export interface ValvePhoto {
  id: string;
  valve_id: string;
  legacy_tag?: string | null;
  blob_ref: string;
}

export interface MaintenanceRecord {
  id: string;
  valve_id: string;
  valve_tag_snapshot: string;
  performed_at: string;
  technician: string;
  maintenance_type: "preventive" | "corrective";
  service: string;
  notes?: string | null;
  signature_ref?: string | null;
  kit_changed: boolean;
  stock_consumed: boolean;
  stock_consumption_pending: boolean;
}

export interface ServiceOrder {
  id: string;
  zone: string;
  valve_id?: string | null;
  valve_tag_snapshot?: string | null;
  description: string;
  priority: "low" | "normal" | "high" | "urgent";
  status: "pending" | "in_progress" | "completed";
  created_by: string;
  technician?: string | null;
  scheduled_for?: string | null;
}

export interface StockItem {
  id: string;
  reference: string;
  quantity: number;
  minimum_quantity: number;
  manufacturer?: string | null;
  location?: string | null;
}

export interface RestockRequest {
  id: string;
  reference: string;
  reason: string;
  requested_by: string;
  status: "pending" | "approved" | "rejected";
  reviewed_by?: string | null;
  reviewed_at_ms?: number | null;
}

export interface Supplier {
  id: string;
  name: string;
  contact: string;
  email?: string | null;
  website?: string | null;
  notes?: string | null;
}

export interface AuditEvent {
  id: string;
  actor: string;
  role: Role;
  operation: string;
  aggregate: string;
  aggregate_id: string;
  description?: string | null;
  trace_id?: string | null;
  before_json?: string | null;
  after_json?: string | null;
  result: string;
  created_at_ms: number;
}

export interface ListResult<T> {
  items: T[];
  schema_version: number;
  source: "appcore" | "unavailable";
}

export interface AuditListResult extends ListResult<AuditEvent> {
  total: number;
  page: number;
  page_size: number;
  operations: string[];
  actors: string[];
  aggregates: string[];
}

export interface ValveListResult extends ListResult<Valve> {
  total: number;
  page: number;
  page_size: number;
  facets: { zones: string[]; valve_types: string[] };
}

export interface OverviewResult {
  schema_version: number;
  source: "appcore" | "unavailable";
  valves: { total: number; ok: number; warning: number; critical: number };
  orders: { open: number; in_progress: number; completed: number };
  stock: { low: number; total: number };
  recent_maintenance: MaintenanceRecord[];
  upcoming_orders: ServiceOrder[];
}

export interface ReportResult {
  schema_version: number;
  source: "appcore" | "unavailable";
  generated_at_ms: number;
  overview: Omit<OverviewResult, "source">;
  by_zone: Array<{ zone: string; total: number; critical: number; warning: number }>;
  critical_valves: Array<Pick<Valve, "id" | "tag" | "zone" | "last_maintenance_at" | "health">>;
  recent_maintenance: MaintenanceRecord[];
}

export interface CommandResult {
  accepted: boolean;
  message?: string | null;
}
