use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Valve {
    pub id: String,
    pub tag: String,
    pub tag_normalized: String,
    pub zone: String,
    pub manufacturer: Option<String>,
    pub serial: Option<String>,
    pub kit_reference: Option<String>,
    pub seat: Option<String>,
    pub dn: Option<String>,
    pub valve_type: Option<String>,
    pub actuator: Option<String>,
    pub manufactured_at: Option<String>,
    pub last_kit_changed_at: Option<String>,
    pub last_maintenance_at: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceType {
    Preventive,
    Corrective,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenanceRecord {
    pub id: String,
    pub valve_id: String,
    pub valve_tag_snapshot: String,
    pub performed_at: String,
    pub technician: String,
    pub maintenance_type: MaintenanceType,
    pub service: String,
    pub notes: Option<String>,
    pub signature_ref: Option<String>,
    pub kit_changed: bool,
    pub kit_reference_snapshot: Option<String>,
    pub stock_consumed: bool,
    pub stock_consumption_pending: bool,
    pub idempotency_key: String,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceOrderStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceOrder {
    pub id: String,
    pub zone: String,
    pub valve_id: Option<String>,
    pub valve_tag_snapshot: Option<String>,
    pub description: String,
    pub priority: String,
    pub status: ServiceOrderStatus,
    pub created_by: String,
    pub technician: Option<String>,
    pub scheduled_for: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestockStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestockRequest {
    pub id: String,
    pub reference: String,
    pub reason: String,
    pub requested_by: String,
    pub status: RestockStatus,
    pub reviewed_by: Option<String>,
    pub reviewed_at_ms: Option<u64>,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockItem {
    pub id: String,
    pub reference: String,
    pub reference_normalized: String,
    pub quantity: u32,
    pub minimum_quantity: u32,
    pub manufacturer: Option<String>,
    pub location: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StockMovementKind {
    Receipt,
    Consumption,
    Correction,
    Migration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockMovement {
    pub id: String,
    pub stock_item_id: String,
    pub kind: StockMovementKind,
    pub delta: i32,
    pub balance_after: u32,
    pub reason: String,
    pub actor: String,
    pub idempotency_key: String,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Supplier {
    pub id: String,
    pub name: String,
    pub contact: String,
    pub email: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValvePhoto {
    pub id: String,
    pub valve_id: String,
    pub legacy_tag: Option<String>,
    pub blob_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub actor: String,
    pub role: String,
    pub operation: String,
    pub aggregate: String,
    pub aggregate_id: String,
    pub description: Option<String>,
    pub trace_id: Option<String>,
    pub before_json: Option<String>,
    pub after_json: Option<String>,
    pub result: String,
    pub created_at_ms: u64,
}
