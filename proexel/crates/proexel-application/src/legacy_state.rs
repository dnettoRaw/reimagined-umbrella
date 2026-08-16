use std::collections::BTreeMap;

use proexel_domain::{
    derive_machine_status, AuditEvent, ComplexityLevel, CustomFieldDefinition, CustomFieldType,
    InspectionStatus, InstalledComponent, ItemCategory, ItemInspection, Machine, MachineItem,
    MaintenanceGuide, OperationalStatus, PhotoAsset, PhotoOwnerType, PhotoPurpose,
    ReplacementSpecification, RestockRequest, ServiceOrder, ServiceOrderPriority,
    ServiceOrderStatus, ServiceOrderTask, ServiceOrderTaskStatus, StockItem, StockMovement,
    Supplier, UserAccount,
};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    legacy_support::{
        category_snapshot, clean, date_to_ms, default_repair_level, hash, insert_optional,
        item_snapshot, machine_snapshot,
    },
    ApplicationState, SCHEMA_VERSION,
};

const MIGRATED_CATEGORY_ID: &str = "migrated-category-valve";

#[derive(Deserialize)]
struct LegacyStateV1 {
    #[serde(default)]
    valves: Vec<LegacyValve>,
    #[serde(default)]
    maintenance_records: Vec<LegacyMaintenance>,
    #[serde(default)]
    service_orders: Vec<LegacyOrder>,
    #[serde(default)]
    restock_requests: Vec<RestockRequest>,
    #[serde(default)]
    stock_items: Vec<StockItem>,
    #[serde(default)]
    stock_movements: Vec<StockMovement>,
    #[serde(default)]
    suppliers: Vec<Supplier>,
    #[serde(default)]
    valve_photos: Vec<LegacyPhoto>,
    #[serde(default)]
    user_accounts: Vec<UserAccount>,
    #[serde(default)]
    audit_events: Vec<AuditEvent>,
}

#[derive(Clone, Deserialize)]
struct LegacyValve {
    id: String,
    tag: String,
    tag_normalized: String,
    zone: String,
    manufacturer: Option<String>,
    serial: Option<String>,
    kit_reference: Option<String>,
    seat: Option<String>,
    dn: Option<String>,
    valve_type: Option<String>,
    actuator: Option<String>,
    manufactured_at: Option<String>,
    last_kit_changed_at: Option<String>,
    last_maintenance_at: Option<String>,
    created_at_ms: u64,
    updated_at_ms: u64,
}

#[derive(Deserialize)]
struct LegacyMaintenance {
    id: String,
    valve_id: String,
    valve_tag_snapshot: String,
    performed_at: String,
    technician: String,
    maintenance_type: Value,
    service: String,
    notes: Option<String>,
    signature_ref: Option<String>,
    kit_changed: bool,
    kit_reference_snapshot: Option<String>,
    stock_consumed: bool,
    stock_consumption_pending: bool,
    created_at_ms: u64,
}

#[derive(Deserialize)]
struct LegacyOrder {
    id: String,
    zone: String,
    valve_id: Option<String>,
    valve_tag_snapshot: Option<String>,
    description: String,
    priority: ServiceOrderPriority,
    status: LegacyOrderStatus,
    created_by: String,
    technician: Option<String>,
    scheduled_for: Option<String>,
    created_at_ms: u64,
    updated_at_ms: u64,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LegacyOrderStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Deserialize)]
struct LegacyPhoto {
    id: String,
    valve_id: String,
    legacy_tag: Option<String>,
    blob_ref: String,
}

impl ApplicationState {
    pub fn decode_persisted(bytes: &[u8]) -> Result<(Self, bool), String> {
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|error| format!("storage_decode_failed: {error}"))?;
        let schema_version = value
            .get("schema_version")
            .and_then(Value::as_u64)
            .unwrap_or(1) as u32;
        if schema_version >= SCHEMA_VERSION {
            let state = serde_json::from_value(value)
                .map_err(|error| format!("storage_decode_failed: {error}"))?;
            return Ok((state, false));
        }
        if schema_version != 1 {
            return Err(format!("unsupported_schema_version:{schema_version}"));
        }
        let legacy: LegacyStateV1 = serde_json::from_value(value)
            .map_err(|error| format!("storage_v1_decode_failed: {error}"))?;
        Ok((migrate_v1(legacy), true))
    }
}

fn migrate_v1(legacy: LegacyStateV1) -> ApplicationState {
    let mut state = ApplicationState {
        schema_version: SCHEMA_VERSION,
        restock_requests: legacy.restock_requests,
        stock_items: legacy.stock_items,
        stock_movements: legacy.stock_movements,
        suppliers: legacy.suppliers,
        user_accounts: legacy.user_accounts,
        audit_events: legacy.audit_events,
        ..ApplicationState::default()
    };
    for user in &mut state.user_accounts {
        user.maximum_repair_level = default_repair_level(&user.role);
    }
    if !legacy.valves.is_empty() {
        state.item_categories.push(migrated_category());
    }
    let mut machine_by_zone = BTreeMap::<String, String>::new();
    for zone in legacy
        .valves
        .iter()
        .map(|item| item.zone.trim())
        .chain(legacy.service_orders.iter().map(|order| order.zone.trim()))
        .filter(|zone| !zone.is_empty())
    {
        let normalized = zone.to_uppercase();
        if machine_by_zone.contains_key(&normalized) {
            continue;
        }
        let id = format!("migrated-machine-{}", hash(&normalized));
        machine_by_zone.insert(normalized, id.clone());
        state.machines.push(Machine {
            id,
            code: format!("MIG-{}", &hash(zone)[..8]),
            code_normalized: format!("MIG-{}", &hash(zone)[..8]),
            name: format!("Migrated assets - {zone}"),
            description: Some("Generated from the schema 1 zone grouping".to_string()),
            zone: zone.to_string(),
            location: None,
            manufacturer: None,
            model: None,
            serial_number: None,
            status: OperationalStatus::Unknown,
            main_photo_id: None,
            active: true,
            created_at_ms: 0,
            updated_at_ms: 0,
        });
    }
    let mut item_by_old_id = BTreeMap::<String, String>::new();
    for old in legacy.valves {
        let Some(machine_id) = machine_by_zone
            .get(&old.zone.trim().to_uppercase())
            .cloned()
        else {
            continue;
        };
        let id = format!("migrated-machine-item-{}", hash(&old.id));
        item_by_old_id.insert(old.id.clone(), id.clone());
        let mut custom_field_values = BTreeMap::new();
        insert_optional(&mut custom_field_values, "seat", old.seat);
        insert_optional(&mut custom_field_values, "dn", old.dn);
        insert_optional(&mut custom_field_values, "valve_type", old.valve_type);
        insert_optional(&mut custom_field_values, "actuator", old.actuator);
        insert_optional(
            &mut custom_field_values,
            "manufactured_at",
            old.manufactured_at,
        );
        insert_optional(
            &mut custom_field_values,
            "last_kit_changed_at",
            old.last_kit_changed_at,
        );
        insert_optional(
            &mut custom_field_values,
            "last_maintenance_at",
            old.last_maintenance_at,
        );
        let installed_component = if old.manufacturer.is_some() || old.serial.is_some() {
            Some(InstalledComponent {
                installation_id: format!("migrated-installation-{}", hash(&old.id)),
                manufacturer: old.manufacturer,
                model: None,
                part_number: None,
                serial_number: old.serial,
                installed_at: None,
                technical_specifications: BTreeMap::new(),
            })
        } else {
            None
        };
        let replacement_specification = ReplacementSpecification {
            part_number: old.kit_reference,
            ..ReplacementSpecification::default()
        };
        let position = state
            .machine_items
            .iter()
            .filter(|item| item.machine_id == machine_id)
            .count() as u32
            + 1;
        state.machine_items.push(MachineItem {
            id,
            machine_id,
            category_id: MIGRATED_CATEGORY_ID.to_string(),
            name: old.tag.clone(),
            code: old.tag,
            code_normalized: old.tag_normalized,
            complexity_level: ComplexityLevel::INTERMEDIATE,
            status: OperationalStatus::Unknown,
            position,
            custom_field_values,
            maintenance_guide_override: None,
            installed_component,
            replacement_specification,
            notes: None,
            active: true,
            removed_at_ms: None,
            created_at_ms: old.created_at_ms,
            updated_at_ms: old.updated_at_ms,
        });
    }
    let category = state.item_categories.first().cloned();
    if let Some(category) = category.as_ref() {
        for old in legacy.maintenance_records {
            let Some(item_id) = item_by_old_id.get(&old.valve_id).cloned() else {
                continue;
            };
            let Some(item) = state.machine_items.iter().find(|item| item.id == item_id) else {
                continue;
            };
            let notes = maintenance_notes(&old);
            let performed_at_ms = date_to_ms(&old.performed_at).unwrap_or(old.created_at_ms);
            state.inspections.push(ItemInspection {
                id: format!("migrated-inspection-{}", hash(&old.id)),
                service_order_task_id: None,
                service_order_id: None,
                machine_id: item.machine_id.clone(),
                machine_item_id: item.id.clone(),
                category_snapshot: category_snapshot(category),
                operator_id: format!("migrated-operator-{}", hash(&old.technician.to_lowercase())),
                operator_name: old.technician,
                status: InspectionStatus::Completed,
                started_at_ms: performed_at_ms,
                completed_at_ms: Some(performed_at_ms),
                status_before: OperationalStatus::Unknown,
                status_after: Some(OperationalStatus::Unknown),
                step_results: Vec::new(),
                findings: Vec::new(),
                photo_ids: Vec::new(),
                notes: Some(notes),
                maintenance_action: clean(Some(old.service)),
            });
        }
        for old in legacy.service_orders {
            let Some(machine_id) = machine_by_zone.get(&old.zone.trim().to_uppercase()) else {
                continue;
            };
            let Some(machine) = state
                .machines
                .iter()
                .find(|machine| &machine.id == machine_id)
                .cloned()
            else {
                continue;
            };
            let selected = if let Some(old_item_id) = old.valve_id.as_ref() {
                item_by_old_id
                    .get(old_item_id)
                    .into_iter()
                    .filter_map(|id| state.machine_items.iter().find(|item| &item.id == id))
                    .collect::<Vec<_>>()
            } else {
                state
                    .machine_items
                    .iter()
                    .filter(|item| item.machine_id == machine.id && item.active)
                    .collect::<Vec<_>>()
            };
            if selected.is_empty() {
                continue;
            }
            let status = map_order_status(old.status);
            let task_status = match status {
                ServiceOrderStatus::Completed => ServiceOrderTaskStatus::Completed,
                ServiceOrderStatus::InProgress => ServiceOrderTaskStatus::InProgress,
                ServiceOrderStatus::Pending | ServiceOrderStatus::Cancelled => {
                    ServiceOrderTaskStatus::Pending
                }
            };
            let tasks = selected
                .into_iter()
                .map(|item| ServiceOrderTask {
                    id: format!("migrated-task-{}-{}", hash(&old.id), hash(&item.id)),
                    machine_item_id: item.id.clone(),
                    item_snapshot: item_snapshot(item, category),
                    complexity_snapshot: item.complexity_level,
                    assigned_operator_id: None,
                    status: task_status,
                    started_at_ms: (task_status != ServiceOrderTaskStatus::Pending)
                        .then_some(old.updated_at_ms),
                    completed_at_ms: (task_status == ServiceOrderTaskStatus::Completed)
                        .then_some(old.updated_at_ms),
                    inspection_id: None,
                })
                .collect();
            let description = [
                Some(old.description),
                clean(old.technician).map(|value| format!("Previous technician: {value}")),
                clean(old.valve_tag_snapshot).map(|value| format!("Previous item: {value}")),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n");
            state.service_orders.push(ServiceOrder {
                id: format!("migrated-order-{}", hash(&old.id)),
                machine_id: machine.id.clone(),
                machine_snapshot: machine_snapshot(&machine),
                description,
                priority: old.priority,
                status,
                created_by: old.created_by,
                scheduled_for: old.scheduled_for,
                tasks,
                created_at_ms: old.created_at_ms,
                started_at_ms: (!matches!(status, ServiceOrderStatus::Pending))
                    .then_some(old.updated_at_ms),
                completed_at_ms: (status == ServiceOrderStatus::Completed)
                    .then_some(old.updated_at_ms),
                updated_at_ms: old.updated_at_ms,
            });
        }
    }
    for old in legacy.valve_photos {
        let Some(owner_id) = item_by_old_id.get(&old.valve_id).cloned() else {
            continue;
        };
        state.photos.push(PhotoAsset {
            id: format!("migrated-photo-{}", hash(&old.id)),
            owner_type: PhotoOwnerType::MachineItem,
            owner_id,
            purpose: PhotoPurpose::Reference,
            blob_ref: old.blob_ref,
            description: old
                .legacy_tag
                .map(|tag| format!("Migrated reference photo for {tag}")),
            created_by: "schema-migration".to_string(),
            created_at_ms: 0,
        });
    }
    for machine in &mut state.machines {
        machine.status = derive_machine_status(
            state
                .machine_items
                .iter()
                .filter(|item| item.active && item.machine_id == machine.id)
                .map(|item| &item.status),
        );
    }
    state.audit_events.push(AuditEvent {
        id: "audit-schema-v1-v2".to_string(),
        actor: "schema-migration".to_string(),
        role: "system".to_string(),
        operation: "proexel.schema.migrate_v1_v2".to_string(),
        aggregate: "application_state".to_string(),
        aggregate_id: "schema-2".to_string(),
        description: Some("Migrated persisted state from schema 1 to schema 2".to_string()),
        trace_id: None,
        before_json: Some("{\"schema_version\":1}".to_string()),
        after_json: Some("{\"schema_version\":2}".to_string()),
        result: "success".to_string(),
        created_at_ms: 0,
    });
    state
}

fn migrated_category() -> ItemCategory {
    let fields = [
        ("seat", "Seat", CustomFieldType::Text),
        ("dn", "Nominal diameter", CustomFieldType::Text),
        ("valve_type", "Type", CustomFieldType::Text),
        ("actuator", "Actuator", CustomFieldType::Text),
        ("manufactured_at", "Manufactured at", CustomFieldType::Date),
        (
            "last_kit_changed_at",
            "Last kit change",
            CustomFieldType::Date,
        ),
        (
            "last_maintenance_at",
            "Last maintenance",
            CustomFieldType::Date,
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (key, label, field_type))| CustomFieldDefinition {
        id: format!("migrated-field-{key}"),
        key: key.to_string(),
        label: label.to_string(),
        field_type,
        required: false,
        unit: None,
        options: Vec::new(),
        minimum: None,
        maximum: None,
        order: index as u32,
    })
    .collect();
    ItemCategory {
        id: MIGRATED_CATEGORY_ID.to_string(),
        code: "VALVE".to_string(),
        code_normalized: "VALVE".to_string(),
        name: "Valve".to_string(),
        description: Some("Category migrated from schema 1".to_string()),
        icon: Some("circle-dot".to_string()),
        default_complexity_level: ComplexityLevel::INTERMEDIATE,
        maintenance_guide: MaintenanceGuide::default(),
        custom_field_definitions: fields,
        recommended_parts: Vec::new(),
        active: true,
        created_at_ms: 0,
        updated_at_ms: 0,
    }
}

fn maintenance_notes(old: &LegacyMaintenance) -> String {
    [
        old.notes.clone(),
        Some(format!("Previous type: {}", old.maintenance_type)),
        Some(format!("Previous item: {}", old.valve_tag_snapshot)),
        old.kit_changed
            .then(|| "Previous kit replacement: yes".to_string()),
        old.kit_reference_snapshot
            .clone()
            .map(|value| format!("Previous kit: {value}")),
        old.signature_ref
            .clone()
            .map(|value| format!("Previous signature: {value}")),
        old.stock_consumed
            .then(|| "Previous stock consumption: completed".to_string()),
        old.stock_consumption_pending
            .then(|| "Previous stock consumption: pending".to_string()),
        Some(format!("Previous performed at: {}", old.performed_at)),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("\n")
}

fn map_order_status(status: LegacyOrderStatus) -> ServiceOrderStatus {
    match status {
        LegacyOrderStatus::Pending => ServiceOrderStatus::Pending,
        LegacyOrderStatus::InProgress => ServiceOrderStatus::InProgress,
        LegacyOrderStatus::Completed => ServiceOrderStatus::Completed,
    }
}
