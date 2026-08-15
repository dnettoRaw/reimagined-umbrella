use proexel_application::{ApplicationState, SCHEMA_VERSION};
use proexel_domain::PhotoOwnerType;
use serde_json::json;

use crate::{migrate_bundle, migration_support::date_to_ms, LegacyBundle};

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
