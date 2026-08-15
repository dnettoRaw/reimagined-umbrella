use std::collections::BTreeMap;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct ComplexityLevel(u8);

impl ComplexityLevel {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 5;

    pub fn new(value: u8) -> Result<Self, &'static str> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err("complexity_level_out_of_range")
        }
    }

    pub fn get(self) -> u8 {
        self.0
    }
}

impl Default for ComplexityLevel {
    fn default() -> Self {
        Self(3)
    }
}

impl<'de> Deserialize<'de> for ComplexityLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationalStatus {
    #[default]
    Unknown,
    Ok,
    Attention,
    Critical,
    MaintenanceRequired,
    UnderMaintenance,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomFieldType {
    Text,
    Number,
    Boolean,
    Choice,
    Date,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomFieldDefinition {
    pub id: String,
    pub key: String,
    pub label: String,
    pub field_type: CustomFieldType,
    #[serde(default)]
    pub required: bool,
    pub unit: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub order: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuideStepType {
    Confirmation,
    Boolean,
    Choice,
    Numeric,
    Text,
    Photo,
    Measurement,
    Information,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedValue {
    pub unit: Option<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceGuideStep {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub instructions: String,
    pub step_type: GuideStepType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub reference_photo_ids: Vec<String>,
    pub safety_warning: Option<String>,
    pub expected_value: Option<ExpectedValue>,
    #[serde(default)]
    pub options: Vec<String>,
    pub order: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceGuide {
    pub version: u32,
    #[serde(default)]
    pub steps: Vec<MaintenanceGuideStep>,
}

impl Default for MaintenanceGuide {
    fn default() -> Self {
        Self {
            version: 1,
            steps: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecommendedPart {
    pub manufacturer: Option<String>,
    pub part_number: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemCategory {
    pub id: String,
    pub code: String,
    pub code_normalized: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub default_complexity_level: ComplexityLevel,
    pub maintenance_guide: MaintenanceGuide,
    #[serde(default)]
    pub custom_field_definitions: Vec<CustomFieldDefinition>,
    #[serde(default)]
    pub recommended_parts: Vec<RecommendedPart>,
    pub active: bool,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub code: String,
    pub code_normalized: String,
    pub name: String,
    pub description: Option<String>,
    pub zone: String,
    pub location: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub status: OperationalStatus,
    pub main_photo_id: Option<String>,
    pub active: bool,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstalledComponent {
    pub installation_id: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub part_number: Option<String>,
    pub serial_number: Option<String>,
    pub installed_at: Option<String>,
    #[serde(default)]
    pub technical_specifications: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquivalentPart {
    pub manufacturer: Option<String>,
    pub part_number: String,
    pub model: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ReplacementSpecification {
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    #[serde(default)]
    pub technical_specifications: BTreeMap<String, Value>,
    pub compatibility_notes: Option<String>,
    #[serde(default)]
    pub equivalent_parts: Vec<EquivalentPart>,
    pub supplier_reference: Option<String>,
    #[serde(default)]
    pub photo_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MachineItem {
    pub id: String,
    pub machine_id: String,
    pub category_id: String,
    pub name: String,
    pub code: String,
    pub code_normalized: String,
    pub complexity_level: ComplexityLevel,
    pub status: OperationalStatus,
    pub position: u32,
    pub location_description: Option<String>,
    #[serde(default)]
    pub custom_field_values: BTreeMap<String, Value>,
    pub installed_component: Option<InstalledComponent>,
    pub replacement_specification: ReplacementSpecification,
    pub notes: Option<String>,
    pub active: bool,
    pub removed_at_ms: Option<u64>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MachineItemReplacement {
    pub id: String,
    pub machine_item_id: String,
    pub previous: Option<InstalledComponent>,
    pub current: InstalledComponent,
    pub reason: String,
    pub replaced_by: String,
    pub replaced_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhotoOwnerType {
    Machine,
    MachineItem,
    GuideStep,
    Inspection,
    Replacement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhotoPurpose {
    Main,
    General,
    Reference,
    Before,
    During,
    After,
    Defect,
    Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotoAsset {
    pub id: String,
    pub owner_type: PhotoOwnerType,
    pub owner_id: String,
    pub purpose: PhotoPurpose,
    pub blob_ref: String,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at_ms: u64,
}

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
    pub location_description: Option<String>,
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
pub struct UserAccount {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub password_hash: String,
    pub pin_hash: Option<String>,
    pub active: bool,
    #[serde(default)]
    pub maximum_repair_level: ComplexityLevel,
    pub auth_version: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
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
