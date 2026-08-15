use std::collections::BTreeMap;

use proexel_application::{can, commands, ApplicationState, Role};
use proexel_domain::{
    ComplexityLevel, Machine, MachineItem, OperationalStatus, ReplacementSpecification,
    ServiceOrderPriority, UserAccount,
};
use serde_json::json;

use crate::{
    asset_queries::machines_payload,
    query_endpoint::{parse_query_payload, query_permission},
    query_support::order_status_counts,
    summary_queries::{reports_payload, users_payload},
};

#[test]
fn malformed_query_payload_is_rejected_explicitly() {
    assert!(parse_query_payload(b"not-json").is_err());
}

fn machine(id: &str, zone: &str, status: OperationalStatus) -> Machine {
    Machine {
        id: id.to_string(),
        code: id.to_uppercase(),
        code_normalized: id.to_uppercase(),
        name: format!("Machine {id}"),
        description: None,
        zone: zone.to_string(),
        location: None,
        manufacturer: None,
        model: None,
        serial_number: None,
        status,
        main_photo_id: None,
        active: true,
        created_at_ms: 0,
        updated_at_ms: 0,
    }
}

fn item(index: usize, machine_id: &str, status: OperationalStatus) -> MachineItem {
    MachineItem {
        id: format!("item-{index}"),
        machine_id: machine_id.to_string(),
        category_id: "category-1".to_string(),
        name: format!("Item {index}"),
        code: format!("I-{index:05}"),
        code_normalized: format!("I-{index:05}"),
        complexity_level: ComplexityLevel::INTERMEDIATE,
        status,
        position: index as u32,
        location_description: None,
        custom_field_values: BTreeMap::new(),
        installed_component: None,
        replacement_specification: ReplacementSpecification::default(),
        notes: None,
        active: true,
        removed_at_ms: None,
        created_at_ms: index as u64,
        updated_at_ms: index as u64,
    }
}

#[test]
fn reports_group_machine_items_by_zone_and_status() {
    let mut state = ApplicationState::default();
    state
        .machines
        .push(machine("machine-1", "Line 1", OperationalStatus::Critical));
    state
        .machine_items
        .push(item(1, "machine-1", OperationalStatus::Critical));

    let report = reports_payload(&state);

    assert_eq!(report["overview"]["machines"]["by_status"]["critical"], 1);
    assert_eq!(report["by_zone"][0]["zone"], "Line 1");
    assert_eq!(report["by_zone"][0]["critical_items"], 1);
    assert_eq!(report["critical_items"][0]["item"]["code"], "I-00001");
}

#[test]
fn query_permissions_cover_every_registered_query() {
    for query in commands::QUERIES {
        assert_ne!(
            query_permission(query),
            Some("unknown"),
            "missing RBAC mapping for {query}"
        );
    }
    assert!(can(
        Role::Tecnico,
        query_permission(commands::LIST_MACHINES).unwrap()
    ));
    assert!(!can(
        Role::Compras,
        query_permission(commands::LIST_MACHINES).unwrap()
    ));
    assert!(can(
        Role::Chefe,
        query_permission(commands::GET_REPORTS).unwrap()
    ));
}

#[test]
fn administrative_user_list_never_exposes_credential_hashes() {
    let mut state = ApplicationState::default();
    state.user_accounts.push(UserAccount {
        id: "u1".to_string(),
        email: "admin@example.com".to_string(),
        name: "Admin".to_string(),
        role: "admin".to_string(),
        password_hash: "scrypt$salt$secret".to_string(),
        pin_hash: Some("scrypt$pin$secret".to_string()),
        active: true,
        maximum_repair_level: ComplexityLevel::EXPERT,
        auth_version: 3,
        created_at_ms: 1,
        updated_at_ms: 2,
    });

    let payload = users_payload(&state);
    let encoded = payload.to_string();
    assert!(!encoded.contains("password_hash"));
    assert!(!encoded.contains("pin_hash"));
    assert!(!encoded.contains("scrypt$"));
    assert_eq!(payload["items"][0]["has_pin"], true);
    assert_eq!(payload["items"][0]["maximum_repair_level"], 5);
}

#[test]
fn production_volume_machine_query_is_paginated_and_bounded() {
    let started = std::time::Instant::now();
    let mut state = ApplicationState::default();
    for index in 0..2_000 {
        let machine_id = format!("machine-{index}");
        state.machines.push(machine(
            &machine_id,
            &format!("Zone {}", index % 20),
            OperationalStatus::Ok,
        ));
        state
            .machine_items
            .push(item(index, &machine_id, OperationalStatus::Ok));
    }
    let machines = machines_payload(&state, &json!({"page": 20, "page_size": 50}));
    assert_eq!(machines["total"], 2_000);
    assert_eq!(machines["items"].as_array().unwrap().len(), 50);
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
}

#[test]
fn order_status_count_includes_cancelled_and_empty_buckets() {
    let state = ApplicationState::default();
    let counts = order_status_counts(&state);
    assert_eq!(counts["pending"], 0);
    assert_eq!(counts["cancelled"], 0);
    let _ = ServiceOrderPriority::Normal;
    let _ = proexel_domain::InspectionStatus::Completed;
}
