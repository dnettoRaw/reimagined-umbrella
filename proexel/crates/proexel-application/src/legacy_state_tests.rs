use proexel_domain::PhotoOwnerType;
use serde_json::json;

use crate::ApplicationState;

#[test]
fn canonical_v1_state_is_migrated_without_losing_operational_records() {
    let bytes = serde_json::to_vec(&json!({
        "schema_version": 1,
        "valves": [{
            "id": "v1", "tag": "FV 1", "tag_normalized": "FV 1", "zone": "A",
            "manufacturer": "Maker", "serial": "S1", "kit_reference": "KIT-1",
            "seat": null, "dn": "50", "valve_type": "butterfly", "actuator": null,
            "manufactured_at": null, "last_kit_changed_at": null,
            "last_maintenance_at": "2026-01-01", "created_at_ms": 1, "updated_at_ms": 2
        }],
        "maintenance_records": [{
            "id": "mt1", "valve_id": "v1", "valve_tag_snapshot": "FV 1",
            "performed_at": "2026-01-01", "technician": "Tech",
            "maintenance_type": "preventive", "service": "Inspect", "notes": null,
            "signature_ref": null, "kit_changed": false, "kit_reference_snapshot": "KIT-1",
            "stock_consumed": false, "stock_consumption_pending": false,
            "idempotency_key": "old", "created_at_ms": 3
        }],
        "service_orders": [{
            "id": "o1", "zone": "A", "valve_id": "v1", "valve_tag_snapshot": "FV 1",
            "description": "Inspect", "priority": "normal", "status": "pending",
            "created_by": "Chief", "technician": null, "scheduled_for": null,
            "created_at_ms": 4, "updated_at_ms": 4
        }],
        "valve_photos": [{"id":"p1","valve_id":"v1","legacy_tag":"FV 1","blob_ref":"p.jpg"}],
        "user_accounts": [{
            "id":"u1","email":"admin@example.com","name":"Admin","role":"admin",
            "password_hash":"hash","pin_hash":null,"active":true,"auth_version":1,
            "created_at_ms":0,"updated_at_ms":0
        }]
    }))
    .unwrap();

    let (state, migrated) = ApplicationState::decode_persisted(&bytes).unwrap();

    assert!(migrated);
    assert_eq!(state.schema_version, 2);
    assert_eq!(state.machines.len(), 1);
    assert_eq!(state.machine_items.len(), 1);
    assert_eq!(state.inspections.len(), 1);
    assert_eq!(state.service_orders[0].tasks.len(), 1);
    assert_eq!(state.photos[0].owner_type, PhotoOwnerType::MachineItem);
    assert_eq!(state.user_accounts[0].maximum_repair_level.get(), 5);
}

#[test]
fn current_state_decodes_without_migration() {
    let bytes = serde_json::to_vec(&ApplicationState::default()).unwrap();
    let (_, migrated) = ApplicationState::decode_persisted(&bytes).unwrap();
    assert!(!migrated);
}
