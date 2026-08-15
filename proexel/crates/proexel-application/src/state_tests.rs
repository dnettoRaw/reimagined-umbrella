use crate::{commands, ApplicationState, Role};

fn command(actor_role: Role, data: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(
        &serde_json::json!({"actor":{"id":"u1","name":"Test","role":actor_role},"data":data}),
    )
    .unwrap()
}

#[test]
fn supplier_rejects_invalid_links() {
    let mut state = ApplicationState::default();
    let invalid = command(
        Role::Admin,
        serde_json::json!({"name":"Supplier","contact":"Person","website":"example.com"}),
    );
    assert_eq!(
        state.execute(commands::CREATE_SUPPLIER, "c1", "idem-1", 1, &invalid),
        Err("supplier_website_invalid".to_string())
    );
}

#[test]
fn user_management_redacts_credentials_and_persists_repair_level() {
    let mut state = ApplicationState::default();
    let create = command(
        Role::Admin,
        serde_json::json!({
            "email":"ADMIN@EXAMPLE.COM",
            "name":"Main Admin",
            "role":"admin",
            "password_hash":format!("scrypt$salt${}", "ab".repeat(32)),
            "maximum_repair_level":5
        }),
    );
    state
        .execute(commands::CREATE_USER, "u1", "user-idem-1", 1, &create)
        .unwrap();
    assert_eq!(state.user_accounts[0].maximum_repair_level.get(), 5);
    let snapshot = state
        .audit_events
        .last()
        .unwrap()
        .after_json
        .as_deref()
        .unwrap();
    assert!(!snapshot.contains("password_hash"));
    assert!(!snapshot.contains("scrypt$"));
}

#[test]
fn last_active_administrator_cannot_be_disabled() {
    let mut state = ApplicationState::default();
    let create = command(
        Role::Admin,
        serde_json::json!({
            "email":"admin@example.com",
            "name":"Main Admin",
            "role":"admin",
            "password_hash":format!("scrypt$salt${}", "ab".repeat(32)),
            "maximum_repair_level":5
        }),
    );
    state
        .execute(commands::CREATE_USER, "u1", "user-idem-1", 1, &create)
        .unwrap();
    let disable = command(
        Role::Admin,
        serde_json::json!({
            "id":"user-u1","email":"admin@example.com","name":"Main Admin",
            "role":"admin","active":false,"maximum_repair_level":5
        }),
    );
    assert_eq!(
        state.execute(commands::UPDATE_USER, "u2", "user-idem-2", 2, &disable),
        Err("last_active_admin_required".to_string())
    );
}
