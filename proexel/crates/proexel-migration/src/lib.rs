use std::collections::BTreeMap;

use proexel_application::ApplicationState;
use proexel_domain::{
    normalize_reference, normalize_tag, AuditEvent, MaintenanceRecord, MaintenanceType,
    RestockRequest, RestockStatus, ServiceOrder, ServiceOrderStatus, StockItem, Supplier, Valve,
    ValvePhoto,
};
use serde::{Deserialize, Serialize};

pub const LEGACY_SOURCE_NAME: &str = "PROEXEL-main";

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
    import_valves(bundle, &mut candidate, now, &mut report);
    import_stock(bundle, &mut candidate, now, &mut report);
    import_maintenance(bundle, &mut candidate, batch_id, now, &mut report);
    import_orders(bundle, &mut candidate, now, &mut report);
    import_restock(bundle, &mut candidate, now, &mut report);
    import_suppliers(bundle, &mut candidate, now, &mut report);
    import_photos(bundle, &mut candidate, &mut report);
    let imported = report.imported_counts.values().sum::<usize>();
    if imported > 0 {
        candidate.audit_events.push(AuditEvent {
            id: format!("audit-migration-{}", hash(batch_id)),
            actor: "migration-tool".to_string(),
            role: "system".to_string(),
            operation: "proexel.migration.import".to_string(),
            aggregate: "migration_batch".to_string(),
            aggregate_id: batch_id.to_string(),
            description: Some(format!("Imported {imported} legacy records")),
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

fn import_valves(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.valves {
        let tag = normalize_tag(&old.tag);
        if tag.is_empty() || old.zone.trim().is_empty() {
            report
                .warnings
                .push(format!("valve skipped: empty TAG or zone ({})", old.tag));
            continue;
        }
        if state.valves.iter().any(|v| v.tag_normalized == tag) {
            continue;
        }
        state.valves.push(Valve {
            id: format!("legacy-valve-{}", hash(&tag)),
            tag: tag.clone(),
            tag_normalized: tag,
            zone: old.zone.trim().to_string(),
            manufacturer: clean(&old.manufacturer),
            serial: clean(&old.serial),
            kit_reference: old
                .kit_reference
                .as_deref()
                .map(normalize_reference)
                .filter(|v| !v.is_empty()),
            seat: clean(&old.seat),
            dn: clean(&old.dn),
            valve_type: clean(&old.valve_type),
            actuator: clean(&old.actuator),
            manufactured_at: clean(&old.manufactured_at),
            last_kit_changed_at: clean(&old.last_kit_changed_at),
            last_maintenance_at: clean(&old.last_maintenance_at),
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "valves");
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
            .any(|v| v.reference_normalized == reference)
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
        inc(report, "stock");
    }
    let refs = state
        .valves
        .iter()
        .filter_map(|v| v.kit_reference.clone())
        .collect::<Vec<_>>();
    for reference in refs {
        if state
            .stock_items
            .iter()
            .any(|v| v.reference_normalized == reference)
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
        inc(report, "stock");
    }
}

fn import_maintenance(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    batch: &str,
    now: u64,
    report: &mut MigrationReport,
) {
    for (index, old) in bundle.maintenance_records.iter().enumerate() {
        let tag = normalize_tag(&old.tag);
        let Some(valve) = state.valves.iter().find(|v| v.tag_normalized == tag) else {
            report.warnings.push(format!(
                "maintenance {index} skipped: valve {tag} not found"
            ));
            continue;
        };
        let id = format!(
            "legacy-maintenance-{}",
            hash(&format!(
                "{tag}|{}|{}|{}",
                old.performed_at, old.technician, old.service
            ))
        );
        if state.maintenance_records.iter().any(|v| v.id == id) {
            continue;
        }
        let kind = if old.maintenance_type.to_lowercase().contains("corr") {
            MaintenanceType::Corrective
        } else {
            MaintenanceType::Preventive
        };
        state.maintenance_records.push(MaintenanceRecord {
            id,
            valve_id: valve.id.clone(),
            valve_tag_snapshot: valve.tag.clone(),
            performed_at: old.performed_at.clone(),
            technician: old.technician.clone(),
            maintenance_type: kind,
            service: old.service.clone(),
            notes: clean(&old.notes),
            signature_ref: clean(&old.signature_ref),
            kit_changed: old.kit_changed,
            kit_reference_snapshot: valve.kit_reference.clone(),
            stock_consumed: false,
            stock_consumption_pending: false,
            idempotency_key: format!("migration-{batch}-{index}"),
            created_at_ms: now,
        });
        inc(report, "maintenance_records");
    }
}

fn import_orders(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.orders {
        let valve = old
            .valve_tag
            .as_deref()
            .map(normalize_tag)
            .and_then(|tag| state.valves.iter().find(|v| v.tag_normalized == tag));
        let id = format!(
            "legacy-order-{}",
            hash(&format!(
                "{}|{}|{}",
                old.zone,
                old.description,
                old.scheduled_for.as_deref().unwrap_or("")
            ))
        );
        if state.service_orders.iter().any(|v| v.id == id) {
            continue;
        }
        state.service_orders.push(ServiceOrder {
            id,
            zone: old.zone.clone(),
            valve_id: valve.map(|v| v.id.clone()),
            valve_tag_snapshot: old.valve_tag.as_deref().map(normalize_tag),
            description: old.description.clone(),
            priority: old.priority.clone(),
            status: order_status(&old.status),
            created_by: old
                .created_by
                .clone()
                .unwrap_or_else(|| "legacy".to_string()),
            technician: clean(&old.technician),
            scheduled_for: clean(&old.scheduled_for),
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "orders");
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
        if state.restock_requests.iter().any(|v| v.id == id) {
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
        if state.suppliers.iter().any(|v| v.id == id) {
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
    report: &mut MigrationReport,
) {
    for old in &bundle.valve_photos {
        let tag = normalize_tag(&old.tag);
        let Some(valve) = state.valves.iter().find(|v| v.tag_normalized == tag) else {
            report
                .warnings
                .push(format!("photo skipped: valve {tag} not found"));
            continue;
        };
        let id = format!("legacy-photo-{}", hash(&format!("{tag}|{}", old.blob_ref)));
        if state.valve_photos.iter().any(|v| v.id == id) {
            continue;
        }
        state.valve_photos.push(ValvePhoto {
            id,
            valve_id: valve.id.clone(),
            legacy_tag: Some(tag),
            blob_ref: old.blob_ref.clone(),
        });
        inc(report, "valve_photos");
    }
}

fn source_counts(b: &LegacyBundle) -> BTreeMap<String, usize> {
    BTreeMap::from([
        ("valves".into(), b.valves.len()),
        ("maintenance_records".into(), b.maintenance_records.len()),
        ("orders".into(), b.orders.len()),
        ("restock_requests".into(), b.restock_requests.len()),
        ("stock".into(), b.stock.len()),
        ("suppliers".into(), b.suppliers.len()),
        ("valve_photos".into(), b.valve_photos.len()),
    ])
}
fn inc(r: &mut MigrationReport, name: &str) {
    *r.imported_counts.entry(name.to_string()).or_default() += 1;
}
fn clean(v: &Option<String>) -> Option<String> {
    v.as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
fn default_priority() -> String {
    "normal".into()
}
fn default_pending() -> String {
    "pending".into()
}
fn order_status(v: &str) -> ServiceOrderStatus {
    match v.trim().to_lowercase().as_str() {
        "andamento" | "in_progress" => ServiceOrderStatus::InProgress,
        "concluida" | "concluída" | "completed" => ServiceOrderStatus::Completed,
        _ => ServiceOrderStatus::Pending,
    }
}
fn restock_status(v: &str) -> RestockStatus {
    match v.trim().to_lowercase().as_str() {
        "aprovada" | "approved" => RestockStatus::Approved,
        "rejeitada" | "rejected" => RestockStatus::Rejected,
        _ => RestockStatus::Pending,
    }
}
fn hash(v: &str) -> String {
    format!("{:016x}", hash_bytes(v.as_bytes()))
}
fn hash_bytes(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_is_deterministic_idempotent_and_supports_dry_run() {
        let bundle: LegacyBundle = serde_json::from_value(serde_json::json!({
            "valves":[{"tag":" fv 1 ","zona":"A","kit":" kit-1 "}],
            "stock":[{"kit":"KIT-1","quantity":2,"minQuantity":1,"brand":"Maker","location":"A-1"}],
            "maintenance_records":[{"tag":"FV 1","date":"2026-01-01","technician":"Tech","type":"preventiva","service":"Inspect","kitChanged":true}],
            "orders":[{"zona":"A","valveTag":"FV 1","observacoes":"Inspect","status":"aberta"}],
            "restock_requests":[{"ref":"KIT-1","description":"Low","status":"pendente"}],
            "suppliers":[{"name":"Supplier","contact":"Person"}],
            "valve_photos":[{"tag":"FV 1","storage_path":"legacy/fv1.jpg"}]
        })).unwrap();
        let mut state = ApplicationState::default();
        assert!(
            migrate_bundle(&bundle, &mut state, "batch-1", 1, true)
                .unwrap()
                .dry_run
        );
        assert!(state.valves.is_empty());
        let first = migrate_bundle(&bundle, &mut state, "batch-1", 1, false).unwrap();
        assert_eq!(first.imported_counts.get("valves"), Some(&1));
        assert_eq!(state.valves[0].tag_normalized, "FV 1");
        assert_eq!(state.stock_items[0].manufacturer.as_deref(), Some("Maker"));
        assert_eq!(state.valve_photos[0].valve_id, state.valves[0].id);
        let second = migrate_bundle(&bundle, &mut state, "batch-1", 1, false).unwrap();
        assert_eq!(second.imported_counts.values().sum::<usize>(), 0);
        assert_eq!(state.audit_events.len(), 1);
    }
}
