export type Role = "admin" | "chefe" | "compras" | "tecnico";
export type ComplexityLevel = 1 | 2 | 3 | 4 | 5;
export type OperationalStatus =
  | "unknown"
  | "ok"
  | "attention"
  | "critical"
  | "maintenance_required"
  | "under_maintenance"
  | "disabled";

export interface UserAccount {
  id: string;
  email: string;
  name: string;
  role: Role;
  active: boolean;
  maximum_repair_level: ComplexityLevel;
  has_pin: boolean;
  auth_version: number;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface IdentityRecord extends Omit<UserAccount, "has_pin"> {
  password_hash: string;
  pin_hash?: string | null;
}

export interface OperatorSummary {
  id: string;
  name: string;
  role: "admin" | "chefe" | "tecnico";
  active: boolean;
  maximum_repair_level: ComplexityLevel;
}

export type CustomFieldType = "text" | "number" | "boolean" | "choice" | "date";

export interface CustomFieldDefinition {
  id: string;
  key: string;
  label: string;
  field_type: CustomFieldType;
  required: boolean;
  unit?: string | null;
  options: string[];
  minimum?: number | null;
  maximum?: number | null;
  order: number;
}

export type GuideStepType =
  | "confirmation"
  | "boolean"
  | "choice"
  | "numeric"
  | "text"
  | "photo"
  | "measurement"
  | "information"
  | "warning";

export interface ExpectedValue {
  unit?: string | null;
  minimum?: number | null;
  maximum?: number | null;
  target?: string | null;
}

export interface MaintenanceGuideStep {
  id: string;
  title: string;
  description?: string | null;
  instructions: string;
  step_type: GuideStepType;
  required: boolean;
  reference_photo_ids: string[];
  safety_warning?: string | null;
  expected_value?: ExpectedValue | null;
  options: string[];
  order: number;
}

export interface MaintenanceGuide {
  version: number;
  steps: MaintenanceGuideStep[];
}

export interface RecommendedPart {
  manufacturer?: string | null;
  part_number: string;
  description?: string | null;
}

export interface ItemCategory {
  id: string;
  code: string;
  code_normalized: string;
  name: string;
  description?: string | null;
  icon?: string | null;
  default_complexity_level: ComplexityLevel;
  maintenance_guide: MaintenanceGuide;
  custom_field_definitions: CustomFieldDefinition[];
  recommended_parts: RecommendedPart[];
  guide_photos: PhotoAsset[];
  active: boolean;
  created_at_ms: number;
  updated_at_ms: number;
}

export interface InstalledComponent {
  installation_id: string;
  manufacturer?: string | null;
  model?: string | null;
  part_number?: string | null;
  serial_number?: string | null;
  installed_at?: string | null;
  technical_specifications: Record<string, unknown>;
}

export interface EquivalentPart {
  manufacturer?: string | null;
  part_number: string;
  model?: string | null;
  notes?: string | null;
}

export interface ReplacementSpecification {
  manufacturer?: string | null;
  part_number?: string | null;
  model?: string | null;
  serial_number?: string | null;
  technical_specifications: Record<string, unknown>;
  compatibility_notes?: string | null;
  equivalent_parts: EquivalentPart[];
  supplier_reference?: string | null;
  photo_ids: string[];
}

export type PhotoOwnerType = "machine" | "machine_item" | "guide_step" | "inspection" | "replacement";
export type PhotoPurpose = "main" | "general" | "reference" | "before" | "during" | "after" | "defect" | "evidence";

export interface PhotoAsset {
  id: string;
  owner_type: PhotoOwnerType;
  owner_id: string;
  purpose: PhotoPurpose;
  blob_ref: string;
  description?: string | null;
  created_by: string;
  created_at_ms: number;
}

export interface MachineItemReplacement {
  id: string;
  machine_item_id: string;
  previous?: InstalledComponent | null;
  current: InstalledComponent;
  reason: string;
  replaced_by: string;
  replaced_at_ms: number;
  photos: PhotoAsset[];
}

export interface MachineItem {
  id: string;
  machine_id: string;
  category_id: string;
  category?: ItemCategory | null;
  name: string;
  code: string;
  code_normalized: string;
  complexity_level: ComplexityLevel;
  status: OperationalStatus;
  position: number;
  custom_field_values: Record<string, unknown>;
  maintenance_guide_override?: MaintenanceGuide | null;
  installed_component?: InstalledComponent | null;
  replacement_specification: ReplacementSpecification;
  notes?: string | null;
  active: boolean;
  removed_at_ms?: number | null;
  created_at_ms: number;
  updated_at_ms: number;
  photos: PhotoAsset[];
  replacement_history: MachineItemReplacement[];
}

export interface Machine {
  id: string;
  code: string;
  code_normalized: string;
  name: string;
  description?: string | null;
  zone: string;
  location?: string | null;
  manufacturer?: string | null;
  model?: string | null;
  serial_number?: string | null;
  status: OperationalStatus;
  main_photo_id?: string | null;
  active: boolean;
  created_at_ms: number;
  updated_at_ms: number;
  items: MachineItem[];
  photos: PhotoAsset[];
}

export interface ItemCategorySnapshot {
  id: string;
  code: string;
  name: string;
  guide_version: number;
  maintenance_guide: MaintenanceGuide;
  guide_reference_photos: PhotoAsset[];
}

export interface MachineItemSnapshot {
  id: string;
  machine_id: string;
  category: ItemCategorySnapshot;
  name: string;
  code: string;
  complexity_level: ComplexityLevel;
  installed_component?: InstalledComponent | null;
}

export type ServiceOrderStatus = "pending" | "in_progress" | "completed" | "cancelled";
export type ServiceOrderTaskStatus = "pending" | "in_progress" | "completed";

export interface ServiceOrderTask {
  id: string;
  machine_item_id: string;
  item_snapshot: MachineItemSnapshot;
  complexity_snapshot: ComplexityLevel;
  assigned_operator_id?: string | null;
  status: ServiceOrderTaskStatus;
  started_at_ms?: number | null;
  completed_at_ms?: number | null;
  inspection_id?: string | null;
}

export interface ServiceOrder {
  id: string;
  machine_id: string;
  machine_snapshot: { id: string; code: string; name: string; zone: string; location?: string | null };
  description: string;
  priority: "low" | "normal" | "high" | "urgent";
  status: ServiceOrderStatus;
  created_by: string;
  scheduled_for?: string | null;
  tasks: ServiceOrderTask[];
  maximum_complexity_level: ComplexityLevel;
  completed_tasks: number;
  created_at_ms: number;
  started_at_ms?: number | null;
  completed_at_ms?: number | null;
  updated_at_ms: number;
}

export interface InspectionStepResult {
  step_id: string;
  value: unknown;
  unit?: string | null;
  photo_ids: string[];
}

export interface InspectionFinding {
  description: string;
  severity: OperationalStatus;
  action_required?: string | null;
}

export interface ItemInspection {
  id: string;
  service_order_task_id?: string | null;
  service_order_id?: string | null;
  machine_id: string;
  machine_item_id: string;
  category_snapshot: ItemCategorySnapshot;
  operator_id: string;
  operator_name: string;
  status: "in_progress" | "completed";
  started_at_ms: number;
  completed_at_ms?: number | null;
  status_before: OperationalStatus;
  status_after?: OperationalStatus | null;
  step_results: InspectionStepResult[];
  findings: InspectionFinding[];
  photo_ids: string[];
  notes?: string | null;
  maintenance_action?: string | null;
  photos: PhotoAsset[];
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
  role: Role | "system";
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
  total?: number;
}

export interface AuditListResult extends ListResult<AuditEvent> {
  total: number;
  page: number;
  page_size: number;
  operations: string[];
  actors: string[];
  aggregates: string[];
}

export interface MachineListResult extends ListResult<Machine> {
  total: number;
  page: number;
  page_size: number;
  facets: { zones: string[] };
}

export interface OverviewResult {
  schema_version: number;
  source: "appcore" | "unavailable";
  machines: { total: number; by_status: Partial<Record<OperationalStatus, number>> };
  machine_items: { total: number; by_status: Partial<Record<OperationalStatus, number>> };
  orders: Record<ServiceOrderStatus, number>;
  stock: { low: number; total: number };
  recent_inspections: ItemInspection[];
  upcoming_orders: ServiceOrder[];
}

export interface ReportResult {
  schema_version: number;
  source: "appcore" | "unavailable";
  generated_at_ms: number;
  overview: Omit<OverviewResult, "source">;
  by_zone: Array<{ zone: string; machines: number; items: number; critical_items: number }>;
  critical_items: Array<{ item: MachineItem; machine?: Machine | null; category?: ItemCategory | null }>;
  recent_inspections: ItemInspection[];
}

export interface CommandResult {
  accepted: boolean;
  message?: string | null;
  resource_id?: string;
}
