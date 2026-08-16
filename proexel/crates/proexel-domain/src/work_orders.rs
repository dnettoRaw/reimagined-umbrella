use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ComplexityLevel, InstalledComponent, MaintenanceGuide, OperationalStatus, PhotoAsset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceOrderStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceOrderPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceOrderTaskStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MachineSnapshot {
    pub id: String,
    pub code: String,
    pub name: String,
    pub zone: String,
    #[serde(default)]
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCategorySnapshot {
    pub id: String,
    pub code: String,
    pub name: String,
    pub guide_version: u32,
    pub maintenance_guide: MaintenanceGuide,
    #[serde(default)]
    pub guide_reference_photos: Vec<PhotoAsset>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MachineItemSnapshot {
    pub id: String,
    pub machine_id: String,
    pub category: ItemCategorySnapshot,
    pub name: String,
    pub code: String,
    pub complexity_level: ComplexityLevel,
    pub installed_component: Option<InstalledComponent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceOrderTask {
    pub id: String,
    pub machine_item_id: String,
    pub item_snapshot: MachineItemSnapshot,
    pub complexity_snapshot: ComplexityLevel,
    pub assigned_operator_id: Option<String>,
    pub status: ServiceOrderTaskStatus,
    pub started_at_ms: Option<u64>,
    pub completed_at_ms: Option<u64>,
    pub inspection_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceOrder {
    pub id: String,
    pub machine_id: String,
    pub machine_snapshot: MachineSnapshot,
    pub description: String,
    pub priority: ServiceOrderPriority,
    pub status: ServiceOrderStatus,
    pub created_by: String,
    pub scheduled_for: Option<String>,
    #[serde(default)]
    pub tasks: Vec<ServiceOrderTask>,
    pub created_at_ms: u64,
    pub started_at_ms: Option<u64>,
    pub completed_at_ms: Option<u64>,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionStatus {
    InProgress,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InspectionStepResult {
    pub step_id: String,
    pub value: Value,
    pub unit: Option<String>,
    #[serde(default)]
    pub photo_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectionFinding {
    pub description: String,
    pub severity: OperationalStatus,
    pub action_required: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemInspection {
    pub id: String,
    pub service_order_task_id: Option<String>,
    pub service_order_id: Option<String>,
    pub machine_id: String,
    pub machine_item_id: String,
    pub category_snapshot: ItemCategorySnapshot,
    pub operator_id: String,
    pub operator_name: String,
    pub status: InspectionStatus,
    pub started_at_ms: u64,
    pub completed_at_ms: Option<u64>,
    pub status_before: OperationalStatus,
    pub status_after: Option<OperationalStatus>,
    #[serde(default)]
    pub step_results: Vec<InspectionStepResult>,
    #[serde(default)]
    pub findings: Vec<InspectionFinding>,
    #[serde(default)]
    pub photo_ids: Vec<String>,
    pub notes: Option<String>,
    pub maintenance_action: Option<String>,
}
