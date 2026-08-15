use serde::{Deserialize, Serialize};

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
