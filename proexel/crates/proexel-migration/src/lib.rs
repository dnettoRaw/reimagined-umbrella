use std::collections::{BTreeMap, BTreeSet};

use proexel_application::{ApplicationState, SCHEMA_VERSION};
use proexel_domain::{
    derive_machine_status, normalize_reference, normalize_tag, AuditEvent, ComplexityLevel,
    CustomFieldDefinition, CustomFieldType, InspectionStatus, InstalledComponent, ItemCategory,
    ItemCategorySnapshot, ItemInspection, Machine, MachineItem, MachineItemSnapshot,
    MachineSnapshot, MaintenanceGuide, OperationalStatus, PhotoAsset, PhotoOwnerType, PhotoPurpose,
    ReplacementSpecification, RestockRequest, RestockStatus, ServiceOrder, ServiceOrderPriority,
    ServiceOrderStatus, ServiceOrderTask, ServiceOrderTaskStatus, StockItem, Supplier,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const LEGACY_SOURCE_NAME: &str = "PROEXEL-main";
const LEGACY_CATEGORY_ID: &str = "legacy-category-valve";

pub fn normalize_legacy_tag(tag: &str) -> String {
    normalize_tag(tag)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LegacyBundle {
    #[serde(default)]
    pub valves: Vec<LegacyValve>,
    #[serde(default)]
    pub maintenance_records: Vec<LegacyMaintenance>,
    #[serde(default)]
    pub orders: Vec<LegacyOrder>,
    #[serde(default)]
    pub restock_requests: Vec<LegacyRestock>,
    #[serde(default)]
    pub stock: Vec<LegacyStock>,
    #[serde(default)]
    pub suppliers: Vec<LegacySupplier>,
    #[serde(default)]
    pub valve_photos: Vec<LegacyPhoto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyValve {
    pub tag: String,
    #[serde(alias = "zona")]
    pub zone: String,
    #[serde(default, alias = "marca")]
    pub manufacturer: Option<String>,
    #[serde(default, alias = "serie")]
    pub serial: Option<String>,
    #[serde(default, alias = "kit")]
    pub kit_reference: Option<String>,
    #[serde(default, alias = "assento")]
    pub seat: Option<String>,
    #[serde(default)]
    pub dn: Option<String>,
    #[serde(default, alias = "tipo")]
    pub valve_type: Option<String>,
    #[serde(default, alias = "atuador")]
    pub actuator: Option<String>,
    #[serde(default, alias = "fabricacao")]
    pub manufactured_at: Option<String>,
    #[serde(default, alias = "ult_kit")]
    pub last_kit_changed_at: Option<String>,
    #[serde(default, alias = "ult_man")]
    pub last_maintenance_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyMaintenance {
    pub tag: String,
    #[serde(alias = "date")]
    pub performed_at: String,
    pub technician: String,
    #[serde(alias = "type")]
    pub maintenance_type: String,
    #[serde(alias = "service")]
    pub service: String,
    #[serde(default, alias = "kitChanged")]
    pub kit_changed: bool,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default, alias = "signature")]
    pub signature_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyOrder {
    #[serde(alias = "zona")]
    pub zone: String,
    #[serde(default, alias = "valveTag", alias = "valve_tag")]
    pub valve_tag: Option<String>,
    #[serde(alias = "observacoes")]
    pub description: String,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default = "default_pending")]
    pub status: String,
    #[serde(default, alias = "createdBy")]
    pub created_by: Option<String>,
    #[serde(default, alias = "tecnico")]
    pub technician: Option<String>,
    #[serde(default, alias = "data_programada")]
    pub scheduled_for: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyRestock {
    #[serde(alias = "ref", alias = "kit")]
    pub reference: String,
    #[serde(alias = "description")]
    pub reason: String,
    #[serde(default, alias = "created_by", alias = "suggested_by")]
    pub requested_by: Option<String>,
    #[serde(default = "default_pending")]
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyStock {
    #[serde(alias = "kit", alias = "ref")]
    pub reference: String,
    #[serde(default)]
    pub quantity: u32,
    #[serde(default, alias = "minQuantity")]
    pub min_quantity: u32,
    #[serde(default)]
    pub manufacturer: Option<String>,
    #[serde(default)]
    pub brand: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacySupplier {
    pub name: String,
    pub contact: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default, alias = "created_by")]
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyPhoto {
    pub tag: String,
    #[serde(alias = "storage_path", alias = "photo_url")]
    pub blob_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    pub source: String,
    pub batch_id: String,
    pub checksum: String,
    pub dry_run: bool,
    pub source_counts: BTreeMap<String, usize>,
    pub imported_counts: BTreeMap<String, usize>,
    pub warnings: Vec<String>,
}

pub fn migrate_bundle(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    batch_id: &str,
    now: u64,
    dry_run: bool,
) -> Result<MigrationReport, String> {
    if batch_id.trim().is_empty() {
        return Err("batch_id_required".to_string());
    }
    let encoded =
        serde_json::to_vec(bundle).map_err(|_| "legacy_bundle_encode_failed".to_string())?;
    let mut candidate = state.clone();
    let mut report = MigrationReport {
        source: LEGACY_SOURCE_NAME.to_string(),
        batch_id: batch_id.to_string(),
        checksum: format!("fnv1a64:{:016x}", hash_bytes(&encoded)),
        dry_run,
        source_counts: source_counts(bundle),
        imported_counts: BTreeMap::new(),
        warnings: Vec::new(),
    };

    ensure_legacy_category(&mut candidate, now, &mut report);
    import_machines(bundle, &mut candidate, now, &mut report);
    import_machine_items(bundle, &mut candidate, now, &mut report);
    import_stock(bundle, &mut candidate, now, &mut report);
    import_inspections(bundle, &mut candidate, now, &mut report);
    import_orders(bundle, &mut candidate, now, &mut report);
    import_restock(bundle, &mut candidate, now, &mut report);
    import_suppliers(bundle, &mut candidate, now, &mut report);
    import_photos(bundle, &mut candidate, now, &mut report);
    refresh_machine_statuses(&mut candidate, now);
    candidate.schema_version = SCHEMA_VERSION;

    let imported = report.imported_counts.values().sum::<usize>();
    if imported > 0 {
        candidate.audit_events.push(AuditEvent {
            id: format!("audit-migration-{}", hash(batch_id)),
            actor: "migration-tool".to_string(),
            role: "system".to_string(),
            operation: "proexel.migration.import".to_string(),
            aggregate: "migration_batch".to_string(),
            aggregate_id: batch_id.to_string(),
            description: Some(format!(
                "Imported {imported} records into schema {SCHEMA_VERSION}"
            )),
            trace_id: Some(batch_id.to_string()),
            before_json: None,
            after_json: serde_json::to_string(&report.imported_counts).ok(),
            result: "success".to_string(),
            created_at_ms: now,
        });
    }
    if !dry_run {
        *state = candidate;
    }
    Ok(report)
}

fn ensure_legacy_category(state: &mut ApplicationState, now: u64, report: &mut MigrationReport) {
    if state
        .item_categories
        .iter()
        .any(|category| category.id == LEGACY_CATEGORY_ID)
    {
        return;
    }
    let definitions = [
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
        id: format!("legacy-field-{key}"),
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
    state.item_categories.push(ItemCategory {
        id: LEGACY_CATEGORY_ID.to_string(),
        code: "VALVE".to_string(),
        code_normalized: "VALVE".to_string(),
        name: "Valve".to_string(),
        description: Some("Category imported from the previous PROEXEL model".to_string()),
        icon: Some("circle-dot".to_string()),
        default_complexity_level: ComplexityLevel::new(3).unwrap(),
        maintenance_guide: MaintenanceGuide::default(),
        custom_field_definitions: definitions,
        recommended_parts: Vec::new(),
        active: true,
        created_at_ms: now,
        updated_at_ms: now,
    });
    inc(report, "item_categories");
}

fn import_machines(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    let zones = bundle
        .valves
        .iter()
        .map(|item| item.zone.trim())
        .chain(bundle.orders.iter().map(|order| order.zone.trim()))
        .filter(|zone| !zone.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    for zone in zones {
        let zone_key = zone.to_uppercase();
        if state.machines.iter().any(|machine| {
            machine.zone.to_uppercase() == zone_key && machine.code.starts_with("LEGACY-")
        }) {
            continue;
        }
        let zone_hash = hash(&zone_key);
        state.machines.push(Machine {
            id: format!("legacy-machine-{zone_hash}"),
            code: format!("LEGACY-{}", &zone_hash[..8]),
            code_normalized: format!("LEGACY-{}", &zone_hash[..8]),
            name: format!("Legacy assets - {zone}"),
            description: Some("Machine generated while migrating legacy zone assets".to_string()),
            zone,
            location: None,
            manufacturer: None,
            model: None,
            serial_number: None,
            status: OperationalStatus::Unknown,
            main_photo_id: None,
            active: true,
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "machines");
    }
}

fn import_machine_items(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.valves {
        let tag = normalize_tag(&old.tag);
        let zone = old.zone.trim();
        if tag.is_empty() || zone.is_empty() {
            report
                .warnings
                .push(format!("asset skipped: empty TAG or zone ({})", old.tag));
            continue;
        }
        let Some(machine_id) = machine_for_zone(state, zone).map(|machine| machine.id.clone())
        else {
            report
                .warnings
                .push(format!("asset {tag} skipped: generated machine not found"));
            continue;
        };
        if state
            .machine_items
            .iter()
            .any(|item| item.machine_id == machine_id && item.code_normalized == tag)
        {
            continue;
        }
        let position = state
            .machine_items
            .iter()
            .filter(|item| item.machine_id == machine_id)
            .count() as u32;
        let mut custom_field_values = BTreeMap::new();
        insert_clean(&mut custom_field_values, "seat", &old.seat);
        insert_clean(&mut custom_field_values, "dn", &old.dn);
        insert_clean(&mut custom_field_values, "valve_type", &old.valve_type);
        insert_clean(&mut custom_field_values, "actuator", &old.actuator);
        insert_clean(
            &mut custom_field_values,
            "manufactured_at",
            &old.manufactured_at,
        );
        insert_clean(
            &mut custom_field_values,
            "last_kit_changed_at",
            &old.last_kit_changed_at,
        );
        insert_clean(
            &mut custom_field_values,
            "last_maintenance_at",
            &old.last_maintenance_at,
        );
        let item_hash = hash(&format!("{}|{}", zone.to_uppercase(), tag));
        let installed_component = if old.manufacturer.is_some() || old.serial.is_some() {
            Some(InstalledComponent {
                installation_id: format!("legacy-installation-{item_hash}"),
                manufacturer: clean(&old.manufacturer),
                model: None,
                part_number: None,
                serial_number: clean(&old.serial),
                installed_at: None,
                technical_specifications: BTreeMap::new(),
            })
        } else {
            None
        };
        let replacement_specification = ReplacementSpecification {
            part_number: old
                .kit_reference
                .as_deref()
                .map(normalize_reference)
                .filter(|value| !value.is_empty()),
            ..ReplacementSpecification::default()
        };
        state.machine_items.push(MachineItem {
            id: format!("legacy-machine-item-{item_hash}"),
            machine_id,
            category_id: LEGACY_CATEGORY_ID.to_string(),
            name: tag.clone(),
            code: tag.clone(),
            code_normalized: tag,
            complexity_level: ComplexityLevel::new(3).unwrap(),
            status: OperationalStatus::Unknown,
            position,
            location_description: None,
            custom_field_values,
            installed_component,
            replacement_specification,
            notes: None,
            active: true,
            removed_at_ms: None,
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "machine_items");
    }
}

fn import_stock(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.stock {
        let reference = normalize_reference(&old.reference);
        if reference.is_empty() {
            report
                .warnings
                .push("stock skipped: empty reference".to_string());
            continue;
        }
        if state
            .stock_items
            .iter()
            .any(|item| item.reference_normalized == reference)
        {
            continue;
        }
        if old.manufacturer.is_none() && old.brand.is_some() && old.location.is_some() {
            report.warnings.push(format!(
                "stock {reference}: brand mapped to manufacturer; location preserved"
            ));
        }
        state.stock_items.push(StockItem {
            id: format!("legacy-stock-{}", hash(&reference)),
            reference: reference.clone(),
            reference_normalized: reference,
            quantity: old.quantity,
            minimum_quantity: old.min_quantity,
            manufacturer: clean(&old.manufacturer).or_else(|| clean(&old.brand)),
            location: clean(&old.location),
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "stock_items");
    }
    let references = state
        .machine_items
        .iter()
        .filter_map(|item| item.replacement_specification.part_number.clone())
        .collect::<BTreeSet<_>>();
    for reference in references {
        if state
            .stock_items
            .iter()
            .any(|item| item.reference_normalized == reference)
        {
            continue;
        }
        state.stock_items.push(StockItem {
            id: format!("legacy-stock-auto-{}", hash(&reference)),
            reference: reference.clone(),
            reference_normalized: reference,
            quantity: 0,
            minimum_quantity: 0,
            manufacturer: None,
            location: None,
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "stock_items");
    }
}

fn import_inspections(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for (index, old) in bundle.maintenance_records.iter().enumerate() {
        let tag = normalize_tag(&old.tag);
        let Some(item) = find_item_by_tag(state, &tag).cloned() else {
            report
                .warnings
                .push(format!("maintenance {index} skipped: item {tag} not found"));
            continue;
        };
        let id = format!(
            "legacy-inspection-{}",
            hash(&format!(
                "{tag}|{}|{}|{}",
                old.performed_at, old.technician, old.service
            ))
        );
        if state
            .inspections
            .iter()
            .any(|inspection| inspection.id == id)
        {
            continue;
        }
        let Some(category) = state
            .item_categories
            .iter()
            .find(|category| category.id == item.category_id)
        else {
            report
                .warnings
                .push(format!("maintenance {index} skipped: category not found"));
            continue;
        };
        let notes = [
            clean(&old.notes),
            Some(format!("Legacy type: {}", old.maintenance_type.trim())),
            old.kit_changed
                .then(|| "Legacy kit replacement: yes".to_string()),
            clean(&old.signature_ref).map(|value| format!("Legacy signature: {value}")),
            Some(format!("Legacy performed at: {}", old.performed_at.trim())),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n");
        let performed_at_ms = date_to_ms(&old.performed_at).unwrap_or(now);
        state.inspections.push(ItemInspection {
            id,
            service_order_task_id: None,
            service_order_id: None,
            machine_id: item.machine_id.clone(),
            machine_item_id: item.id,
            category_snapshot: category_snapshot(category),
            operator_id: format!("legacy-operator-{}", hash(&old.technician.to_lowercase())),
            operator_name: old.technician.trim().to_string(),
            status: InspectionStatus::Completed,
            started_at_ms: performed_at_ms,
            completed_at_ms: Some(performed_at_ms),
            status_before: OperationalStatus::Unknown,
            status_after: Some(OperationalStatus::Unknown),
            step_results: Vec::new(),
            findings: Vec::new(),
            photo_ids: Vec::new(),
            notes: (!notes.is_empty()).then_some(notes),
            maintenance_action: clean(&Some(old.service.clone())),
        });
        inc(report, "inspections");
    }
}

fn import_orders(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.orders {
        let Some(machine) = machine_for_zone(state, &old.zone).cloned() else {
            report.warnings.push(format!(
                "order skipped: machine for zone {} not found",
                old.zone
            ));
            continue;
        };
        let id = format!(
            "legacy-order-{}",
            hash(&format!(
                "{}|{}|{}",
                old.zone,
                old.description,
                old.scheduled_for.as_deref().unwrap_or("")
            ))
        );
        if state.service_orders.iter().any(|order| order.id == id) {
            continue;
        }
        let selected_items = if let Some(tag) = old.valve_tag.as_deref() {
            let normalized = normalize_tag(tag);
            state
                .machine_items
                .iter()
                .filter(|item| {
                    item.machine_id == machine.id
                        && item.code_normalized == normalized
                        && item.active
                })
                .collect::<Vec<_>>()
        } else {
            state
                .machine_items
                .iter()
                .filter(|item| item.machine_id == machine.id && item.active)
                .collect::<Vec<_>>()
        };
        if selected_items.is_empty() {
            report
                .warnings
                .push(format!("order {id} skipped: no target items"));
            continue;
        }
        let status = order_status(&old.status);
        let task_status = match status {
            ServiceOrderStatus::Completed => ServiceOrderTaskStatus::Completed,
            ServiceOrderStatus::InProgress => ServiceOrderTaskStatus::InProgress,
            ServiceOrderStatus::Pending | ServiceOrderStatus::Cancelled => {
                ServiceOrderTaskStatus::Pending
            }
        };
        let tasks = selected_items
            .into_iter()
            .filter_map(|item| {
                let category = state
                    .item_categories
                    .iter()
                    .find(|category| category.id == item.category_id)?;
                Some(ServiceOrderTask {
                    id: format!("{id}-task-{}", hash(&item.id)),
                    machine_item_id: item.id.clone(),
                    item_snapshot: item_snapshot(item, category),
                    complexity_snapshot: item.complexity_level,
                    assigned_operator_id: None,
                    status: task_status,
                    started_at_ms: (task_status != ServiceOrderTaskStatus::Pending).then_some(now),
                    completed_at_ms: (task_status == ServiceOrderTaskStatus::Completed)
                        .then_some(now),
                    inspection_id: None,
                })
            })
            .collect::<Vec<_>>();
        let description = match clean(&old.technician) {
            Some(technician) => format!(
                "{}\nLegacy technician: {technician}",
                old.description.trim()
            ),
            None => old.description.trim().to_string(),
        };
        state.service_orders.push(ServiceOrder {
            id,
            machine_id: machine.id.clone(),
            machine_snapshot: machine_snapshot(&machine),
            description,
            priority: order_priority(&old.priority),
            status,
            created_by: old
                .created_by
                .clone()
                .unwrap_or_else(|| "legacy".to_string()),
            scheduled_for: clean(&old.scheduled_for),
            tasks,
            created_at_ms: now,
            started_at_ms: (!matches!(status, ServiceOrderStatus::Pending)).then_some(now),
            completed_at_ms: (status == ServiceOrderStatus::Completed).then_some(now),
            updated_at_ms: now,
        });
        inc(report, "service_orders");
    }
}

fn import_restock(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.restock_requests {
        let reference = normalize_reference(&old.reference);
        let id = format!(
            "legacy-restock-{}",
            hash(&format!("{}|{}", reference, old.reason))
        );
        if state
            .restock_requests
            .iter()
            .any(|request| request.id == id)
        {
            continue;
        }
        state.restock_requests.push(RestockRequest {
            id,
            reference,
            reason: old.reason.clone(),
            requested_by: old
                .requested_by
                .clone()
                .unwrap_or_else(|| "legacy".to_string()),
            status: restock_status(&old.status),
            reviewed_by: None,
            reviewed_at_ms: None,
            created_at_ms: now,
        });
        inc(report, "restock_requests");
    }
}

fn import_suppliers(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.suppliers {
        if old.name.trim().is_empty() || old.contact.trim().is_empty() {
            report.warnings.push(format!(
                "supplier skipped: name/contact required ({})",
                old.name
            ));
            continue;
        }
        let id = format!("legacy-supplier-{}", hash(&old.name.trim().to_uppercase()));
        if state.suppliers.iter().any(|supplier| supplier.id == id) {
            continue;
        }
        state.suppliers.push(Supplier {
            id,
            name: old.name.trim().to_string(),
            contact: old.contact.trim().to_string(),
            email: clean(&old.email),
            website: clean(&old.website),
            notes: clean(&old.notes),
            created_by: old
                .created_by
                .clone()
                .unwrap_or_else(|| "legacy".to_string()),
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "suppliers");
    }
}

fn import_photos(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.valve_photos {
        let tag = normalize_tag(&old.tag);
        let Some(item) = find_item_by_tag(state, &tag) else {
            report
                .warnings
                .push(format!("photo skipped: item {tag} not found"));
            continue;
        };
        let owner_id = item.id.clone();
        let id = format!("legacy-photo-{}", hash(&format!("{tag}|{}", old.blob_ref)));
        if state.photos.iter().any(|photo| photo.id == id) {
            continue;
        }
        state.photos.push(PhotoAsset {
            id,
            owner_type: PhotoOwnerType::MachineItem,
            owner_id,
            purpose: PhotoPurpose::Reference,
            blob_ref: old.blob_ref.clone(),
            description: Some("Legacy reference photo".to_string()),
            created_by: "migration-tool".to_string(),
            created_at_ms: now,
        });
        inc(report, "photos");
    }
}

fn refresh_machine_statuses(state: &mut ApplicationState, now: u64) {
    for machine in &mut state.machines {
        let statuses = state
            .machine_items
            .iter()
            .filter(|item| item.machine_id == machine.id && item.active)
            .map(|item| &item.status);
        machine.status = derive_machine_status(statuses);
        machine.updated_at_ms = now;
    }
}

fn machine_for_zone<'a>(state: &'a ApplicationState, zone: &str) -> Option<&'a Machine> {
    let zone = zone.trim().to_uppercase();
    state
        .machines
        .iter()
        .find(|machine| machine.zone.to_uppercase() == zone && machine.code.starts_with("LEGACY-"))
}

fn find_item_by_tag<'a>(state: &'a ApplicationState, tag: &str) -> Option<&'a MachineItem> {
    state
        .machine_items
        .iter()
        .find(|item| item.code_normalized == tag)
}

fn category_snapshot(category: &ItemCategory) -> ItemCategorySnapshot {
    ItemCategorySnapshot {
        id: category.id.clone(),
        code: category.code.clone(),
        name: category.name.clone(),
        guide_version: category.maintenance_guide.version,
        maintenance_guide: category.maintenance_guide.clone(),
        guide_reference_photos: Vec::new(),
    }
}

fn item_snapshot(item: &MachineItem, category: &ItemCategory) -> MachineItemSnapshot {
    MachineItemSnapshot {
        id: item.id.clone(),
        machine_id: item.machine_id.clone(),
        category: category_snapshot(category),
        name: item.name.clone(),
        code: item.code.clone(),
        complexity_level: item.complexity_level,
        location_description: item.location_description.clone(),
        installed_component: item.installed_component.clone(),
    }
}

fn machine_snapshot(machine: &Machine) -> MachineSnapshot {
    MachineSnapshot {
        id: machine.id.clone(),
        code: machine.code.clone(),
        name: machine.name.clone(),
        zone: machine.zone.clone(),
    }
}

fn insert_clean(values: &mut BTreeMap<String, Value>, key: &str, value: &Option<String>) {
    if let Some(value) = clean(value) {
        values.insert(key.to_string(), json!(value));
    }
}

fn source_counts(bundle: &LegacyBundle) -> BTreeMap<String, usize> {
    BTreeMap::from([
        ("valves".into(), bundle.valves.len()),
        (
            "maintenance_records".into(),
            bundle.maintenance_records.len(),
        ),
        ("orders".into(), bundle.orders.len()),
        ("restock_requests".into(), bundle.restock_requests.len()),
        ("stock".into(), bundle.stock.len()),
        ("suppliers".into(), bundle.suppliers.len()),
        ("valve_photos".into(), bundle.valve_photos.len()),
    ])
}

fn inc(report: &mut MigrationReport, name: &str) {
    *report.imported_counts.entry(name.to_string()).or_default() += 1;
}

fn clean(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn default_priority() -> String {
    "normal".into()
}

fn default_pending() -> String {
    "pending".into()
}

fn order_status(value: &str) -> ServiceOrderStatus {
    match value.trim().to_lowercase().as_str() {
        "andamento" | "en curso" | "en_cours" | "in_progress" => ServiceOrderStatus::InProgress,
        "concluida" | "concluída" | "completada" | "terminée" | "completed" => {
            ServiceOrderStatus::Completed
        }
        "cancelada" | "annulée" | "cancelled" | "canceled" => ServiceOrderStatus::Cancelled,
        _ => ServiceOrderStatus::Pending,
    }
}

fn order_priority(value: &str) -> ServiceOrderPriority {
    match value.trim().to_lowercase().as_str() {
        "low" | "baixa" | "baja" | "faible" => ServiceOrderPriority::Low,
        "high" | "alta" | "haute" => ServiceOrderPriority::High,
        "urgent" | "urgente" => ServiceOrderPriority::Urgent,
        _ => ServiceOrderPriority::Normal,
    }
}

fn restock_status(value: &str) -> RestockStatus {
    match value.trim().to_lowercase().as_str() {
        "aprovada" | "aprobada" | "approuvée" | "approved" => RestockStatus::Approved,
        "rejeitada" | "rechazada" | "rejetée" | "rejected" => RestockStatus::Rejected,
        _ => RestockStatus::Pending,
    }
}

fn date_to_ms(value: &str) -> Option<u64> {
    let date = value.trim().get(..10)?;
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    let days = days_from_civil(year, month, day)?;
    (days >= 0).then_some(days as u64 * 86_400_000)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let shifted_month = month as i32 + if month > 2 { -3 } else { 9 };
    let doy = (153 * shifted_month + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some((era * 146_097 + doe - 719_468) as i64)
}

fn hash(value: &str) -> String {
    format!("{:016x}", hash_bytes(value.as_bytes()))
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_is_deterministic_idempotent_and_supports_dry_run() {
        let bundle: LegacyBundle = serde_json::from_value(serde_json::json!({
            "valves":[{"tag":" fv 1 ","zona":"A","kit":" kit-1 ","marca":"Maker","serie":"S-1"}],
            "stock":[{"kit":"KIT-1","quantity":2,"minQuantity":1,"brand":"Maker","location":"A-1"}],
            "maintenance_records":[{"tag":"FV 1","date":"2026-01-01","technician":"Tech","type":"preventiva","service":"Inspect","kitChanged":true}],
            "orders":[{"zona":"A","valveTag":"FV 1","observacoes":"Inspect","status":"aberta"}],
            "restock_requests":[{"ref":"KIT-1","description":"Low","status":"pendente"}],
            "suppliers":[{"name":"Supplier","contact":"Person"}],
            "valve_photos":[{"tag":"FV 1","storage_path":"legacy/fv1.jpg"}]
        }))
        .unwrap();
        let mut state = ApplicationState::default();
        assert!(
            migrate_bundle(&bundle, &mut state, "batch-1", 1, true)
                .unwrap()
                .dry_run
        );
        assert!(state.machine_items.is_empty());

        let first = migrate_bundle(&bundle, &mut state, "batch-1", 1, false).unwrap();
        assert_eq!(first.imported_counts.get("item_categories"), Some(&1));
        assert_eq!(first.imported_counts.get("machine_items"), Some(&1));
        assert_eq!(state.item_categories[0].code, "VALVE");
        assert_eq!(state.machines.len(), 1);
        assert_eq!(state.machine_items[0].code_normalized, "FV 1");
        assert_eq!(
            state.machine_items[0]
                .installed_component
                .as_ref()
                .and_then(|component| component.serial_number.as_deref()),
            Some("S-1")
        );
        assert_eq!(state.stock_items[0].manufacturer.as_deref(), Some("Maker"));
        assert_eq!(state.inspections.len(), 1);
        assert_eq!(state.service_orders[0].tasks.len(), 1);
        assert_eq!(state.photos[0].owner_id, state.machine_items[0].id);
        assert_eq!(state.photos[0].owner_type, PhotoOwnerType::MachineItem);

        let second = migrate_bundle(&bundle, &mut state, "batch-1", 1, false).unwrap();
        assert_eq!(second.imported_counts.values().sum::<usize>(), 0);
        assert_eq!(state.audit_events.len(), 1);
        assert_eq!(state.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn orders_without_specific_item_snapshot_all_items_in_zone() {
        let bundle: LegacyBundle = serde_json::from_value(json!({
            "valves": [
                {"tag": "A-1", "zone": "A"},
                {"tag": "A-2", "zone": "A"}
            ],
            "orders": [{"zone": "A", "description": "Inspect all"}]
        }))
        .unwrap();
        let mut state = ApplicationState::default();
        migrate_bundle(&bundle, &mut state, "batch-all", 1, false).unwrap();
        assert_eq!(state.service_orders[0].tasks.len(), 2);
        assert_eq!(
            state.service_orders[0].tasks[0].item_snapshot.category.code,
            "VALVE"
        );
    }

    #[test]
    fn legacy_dates_are_preserved_as_inspection_timestamps() {
        assert_eq!(date_to_ms("1970-01-01"), Some(0));
        assert_eq!(date_to_ms("2026-08-13"), Some(20_678 * 86_400_000));
    }
}
