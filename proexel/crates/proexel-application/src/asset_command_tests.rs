use proexel_domain::{ComplexityLevel, OperationalStatus, UserAccount};
use serde_json::Value;

use crate::{commands, ApplicationState, Role};

fn command(actor_id: &str, role: Role, data: Value) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "actor":{"id":actor_id,"name":"Test actor","role":role},
        "data":data
    }))
    .unwrap()
}

fn setup() -> ApplicationState {
    let mut state = ApplicationState::default();
    state.user_accounts.push(UserAccount {
        id: "admin".into(),
        email: "admin@example.com".into(),
        name: "Admin".into(),
        role: "admin".into(),
        password_hash: "scrypt$salt$aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        pin_hash: None,
        active: true,
        maximum_repair_level: ComplexityLevel::EXPERT,
        auth_version: 1,
        created_at_ms: 0,
        updated_at_ms: 0,
    });
    state.user_accounts.push(UserAccount {
        id: "tech-2".into(),
        email: "tech@example.com".into(),
        name: "Tech 2".into(),
        role: "tecnico".into(),
        password_hash: "scrypt$salt$aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        pin_hash: None,
        active: true,
        maximum_repair_level: ComplexityLevel::new(2).unwrap(),
        auth_version: 1,
        created_at_ms: 0,
        updated_at_ms: 0,
    });
    let category = command(
        "admin",
        Role::Admin,
        serde_json::json!({
            "code":"motor","name":"Motor elétrico","default_complexity_level":4,"active":true,
            "custom_field_definitions":[{"id":"power","key":"power","label":"Power","field_type":"number","required":true,"unit":"kW","options":[],"minimum":0,"maximum":100,"order":1}],
            "maintenance_guide":{"version":1,"steps":[{"id":"confirm","title":"Confirm","description":null,"instructions":"Confirm identity","step_type":"confirmation","required":true,"reference_photo_ids":[],"safety_warning":null,"expected_value":null,"options":[],"order":1}]}
        }),
    );
    state
        .execute(commands::CREATE_ITEM_CATEGORY, "cat", "i-cat", 1, &category)
        .unwrap();
    let machine = command(
        "admin",
        Role::Admin,
        serde_json::json!({"code":"M17","name":"Press","zone":"Production","active":true}),
    );
    state
        .execute(commands::CREATE_MACHINE, "m", "i-m", 2, &machine)
        .unwrap();
    state
}

fn add_item(state: &mut ApplicationState, id: &str, complexity: Option<u8>) {
    let mut data = serde_json::json!({
        "machine_id":"machine-m","category_id":"category-cat","name":format!("Motor {id}"),
        "code":id,"custom_field_values":{"power":4.2}
    });
    if let Some(level) = complexity {
        data["complexity_level"] = serde_json::json!(level);
    }
    let payload = command("admin", Role::Admin, data);
    state
        .execute(
            commands::ADD_MACHINE_ITEM,
            id,
            &format!("i-{id}"),
            3,
            &payload,
        )
        .unwrap();
}

#[test]
fn category_machine_and_items_apply_default_and_override_complexity() {
    let mut state = setup();
    add_item(&mut state, "a", None);
    add_item(&mut state, "b", Some(3));
    assert_eq!(state.machine_items[0].complexity_level.get(), 4);
    assert_eq!(state.machine_items[1].complexity_level.get(), 3);
}

#[test]
fn order_all_selection_is_snapshotted_and_does_not_expand() {
    let mut state = setup();
    add_item(&mut state, "a", Some(2));
    let order = command(
        "admin",
        Role::Admin,
        serde_json::json!({"machine_id":"machine-m","all_items":true,"description":"Inspect","priority":"normal"}),
    );
    state
        .execute(commands::CREATE_SERVICE_ORDER, "o", "i-o", 4, &order)
        .unwrap();
    add_item(&mut state, "b", Some(2));
    assert_eq!(state.service_orders[0].tasks.len(), 1);
}

#[test]
fn guide_photo_is_linked_snapshotted_and_protected_from_deletion() {
    let mut state = setup();
    add_item(&mut state, "a", Some(2));
    let add_photo = command(
        "admin",
        Role::Admin,
        serde_json::json!({
            "owner_type":"guide_step",
            "owner_id":"confirm",
            "purpose":"reference",
            "blob_ref":"guide-photos/reference.webp"
        }),
    );
    state
        .execute(commands::ADD_PHOTO, "guide", "i-guide", 4, &add_photo)
        .unwrap();
    assert_eq!(
        state.item_categories[0].maintenance_guide.steps[0].reference_photo_ids,
        vec!["photo-guide"]
    );
    let mismatched_remove = command(
        "admin",
        Role::Admin,
        serde_json::json!({"id":"photo-guide","blob_ref":"guide-photos/other.webp"}),
    );
    assert_eq!(
        state.execute(
            commands::DELETE_PHOTO,
            "delete-mismatch",
            "i-delete-mismatch",
            5,
            &mismatched_remove,
        ),
        Err("photo_ref_mismatch".to_string())
    );

    let order = command(
        "admin",
        Role::Admin,
        serde_json::json!({"machine_id":"machine-m","all_items":true,"description":"Inspect","priority":"normal"}),
    );
    state
        .execute(
            commands::CREATE_SERVICE_ORDER,
            "o-photo",
            "i-o-photo",
            6,
            &order,
        )
        .unwrap();
    assert_eq!(
        state.service_orders[0].tasks[0]
            .item_snapshot
            .category
            .guide_reference_photos[0]
            .blob_ref,
        "guide-photos/reference.webp"
    );

    let remove = command(
        "admin",
        Role::Admin,
        serde_json::json!({"id":"photo-guide","blob_ref":"guide-photos/reference.webp"}),
    );
    assert_eq!(
        state.execute(
            commands::DELETE_PHOTO,
            "delete-guide",
            "i-delete-guide",
            7,
            &remove
        ),
        Err("photo_in_use_by_service_order".to_string())
    );
}

#[test]
fn level_two_operator_cannot_start_level_three_task() {
    let mut state = setup();
    add_item(&mut state, "a", Some(3));
    let order = command(
        "admin",
        Role::Admin,
        serde_json::json!({"machine_id":"machine-m","all_items":true,"description":"Inspect","priority":"normal"}),
    );
    state
        .execute(commands::CREATE_SERVICE_ORDER, "o", "i-o", 4, &order)
        .unwrap();
    let start = command("tech-2", Role::Tecnico, serde_json::json!({"id":"order-o"}));
    assert_eq!(
        state.execute(commands::START_SERVICE_ORDER, "s", "i-s", 5, &start),
        Err("operator_repair_level_insufficient".to_string())
    );
}

#[test]
fn guided_execution_requires_results_and_updates_derived_status() {
    let mut state = setup();
    add_item(&mut state, "a", Some(2));
    let order = command(
        "admin",
        Role::Admin,
        serde_json::json!({"machine_id":"machine-m","all_items":true,"description":"Inspect","priority":"normal","assigned_operator_id":"tech-2"}),
    );
    state
        .execute(commands::CREATE_SERVICE_ORDER, "o", "i-o", 4, &order)
        .unwrap();
    let start_order = command("tech-2", Role::Tecnico, serde_json::json!({"id":"order-o"}));
    state
        .execute(commands::START_SERVICE_ORDER, "so", "i-so", 5, &start_order)
        .unwrap();
    let start = command(
        "tech-2",
        Role::Tecnico,
        serde_json::json!({"order_id":"order-o","task_id":"task-o-0"}),
    );
    state
        .execute(commands::START_INSPECTION, "in", "i-in", 6, &start)
        .unwrap();
    let invalid = command(
        "tech-2",
        Role::Tecnico,
        serde_json::json!({"id":"inspection-in","status_after":"ok","step_results":[]}),
    );
    assert!(state
        .execute(commands::COMPLETE_INSPECTION, "ci", "i-ci", 7, &invalid)
        .unwrap_err()
        .starts_with("inspection_step_required"));
    let complete = command(
        "tech-2",
        Role::Tecnico,
        serde_json::json!({"id":"inspection-in","status_after":"ok","step_results":[{"step_id":"confirm","value":true,"unit":null,"photo_ids":[]}]}),
    );
    state
        .execute(commands::COMPLETE_INSPECTION, "ci2", "i-ci2", 8, &complete)
        .unwrap();
    assert_eq!(state.machine_items[0].status, OperationalStatus::Ok);
    assert_eq!(state.machines[0].status, OperationalStatus::Ok);
}

#[test]
fn replacement_preserves_previous_physical_identity() {
    let mut state = setup();
    add_item(&mut state, "a", Some(2));
    let replace = command(
        "admin",
        Role::Admin,
        serde_json::json!({"id":"machine-item-a","reason":"Failure","installed_component":{"manufacturer":"WEG","serial_number":"NEW"}}),
    );
    state
        .execute(commands::REPLACE_MACHINE_ITEM, "r", "i-r", 9, &replace)
        .unwrap();
    assert_eq!(state.machine_item_replacements.len(), 1);
    assert_eq!(
        state.machine_items[0]
            .installed_component
            .as_ref()
            .and_then(|component| component.serial_number.as_deref()),
        Some("NEW")
    );
}
