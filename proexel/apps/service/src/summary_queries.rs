use std::collections::BTreeMap;

use proexel_application::ApplicationState;
use proexel_domain::{OperationalStatus, ServiceOrderStatus};
use serde_json::{json, Value};

use crate::query_support::{now_ms, order_status_counts, status_counts};

pub(crate) fn overview_payload(state: &ApplicationState) -> Value {
    let machine_statuses = status_counts(state.machines.iter().map(|machine| machine.status));
    let active_items = state.machine_items.iter().filter(|item| item.active);
    let item_statuses = status_counts(active_items.clone().map(|item| item.status));
    let order_counts = order_status_counts(state);
    let low_stock = state
        .stock_items
        .iter()
        .filter(|item| item.quantity <= item.minimum_quantity)
        .count();
    let mut recent_inspections = state.inspections.iter().collect::<Vec<_>>();
    recent_inspections.sort_by_key(|inspection| std::cmp::Reverse(inspection.started_at_ms));
    let mut upcoming_orders = state
        .service_orders
        .iter()
        .filter(|order| {
            order.scheduled_for.is_some()
                && !matches!(
                    order.status,
                    ServiceOrderStatus::Completed | ServiceOrderStatus::Cancelled
                )
        })
        .collect::<Vec<_>>();
    upcoming_orders.sort_by(|left, right| left.scheduled_for.cmp(&right.scheduled_for));
    json!({
        "schema_version": state.schema_version,
        "machines": {"total": state.machines.len(), "by_status": machine_statuses},
        "machine_items": {"total": active_items.count(), "by_status": item_statuses},
        "orders": order_counts,
        "stock": {"low": low_stock, "total": state.stock_items.len()},
        "recent_inspections": recent_inspections.into_iter().take(5).collect::<Vec<_>>(),
        "upcoming_orders": upcoming_orders.into_iter().take(5).collect::<Vec<_>>(),
    })
}

pub(crate) fn reports_payload(state: &ApplicationState) -> Value {
    let overview = overview_payload(state);
    let mut by_zone = BTreeMap::<String, (usize, usize, usize)>::new();
    for machine in &state.machines {
        let row = by_zone.entry(machine.zone.clone()).or_default();
        row.0 += 1;
        for item in state
            .machine_items
            .iter()
            .filter(|item| item.active && item.machine_id == machine.id)
        {
            row.1 += 1;
            if matches!(
                item.status,
                OperationalStatus::Critical | OperationalStatus::MaintenanceRequired
            ) {
                row.2 += 1;
            }
        }
    }
    let zones = by_zone
        .into_iter()
        .map(|(zone, (machines, items, critical_items))| {
            json!({
                "zone": zone,
                "machines": machines,
                "items": items,
                "critical_items": critical_items,
            })
        })
        .collect::<Vec<_>>();
    let critical_items = state
        .machine_items
        .iter()
        .filter(|item| {
            item.active
                && matches!(
                    item.status,
                    OperationalStatus::Critical | OperationalStatus::MaintenanceRequired
                )
        })
        .map(|item| {
            json!({
                "item": item,
                "machine": state.machines.iter().find(|machine| machine.id == item.machine_id),
                "category": state.item_categories.iter().find(|category| category.id == item.category_id),
            })
        })
        .collect::<Vec<_>>();
    let mut recent_inspections = state.inspections.iter().collect::<Vec<_>>();
    recent_inspections.sort_by_key(|inspection| std::cmp::Reverse(inspection.started_at_ms));
    json!({
        "schema_version": state.schema_version,
        "generated_at_ms": now_ms(),
        "overview": overview,
        "by_zone": zones,
        "critical_items": critical_items,
        "recent_inspections": recent_inspections,
    })
}

pub(crate) fn users_payload(state: &ApplicationState) -> Value {
    let items = state
        .user_accounts
        .iter()
        .map(|user| {
            json!({
                "id": user.id,
                "email": user.email,
                "name": user.name,
                "role": user.role,
                "active": user.active,
                "maximum_repair_level": user.maximum_repair_level,
                "has_pin": user.pin_hash.is_some(),
                "auth_version": user.auth_version,
                "created_at_ms": user.created_at_ms,
                "updated_at_ms": user.updated_at_ms,
            })
        })
        .collect::<Vec<_>>();
    json!({"items": items, "schema_version": state.schema_version})
}

pub(crate) fn operators_payload(state: &ApplicationState) -> Value {
    let items = state
        .user_accounts
        .iter()
        .filter(|user| user.active && matches!(user.role.as_str(), "admin" | "chefe" | "tecnico"))
        .map(|user| {
            json!({
                "id": user.id,
                "name": user.name,
                "role": user.role,
                "active": user.active,
                "maximum_repair_level": user.maximum_repair_level,
            })
        })
        .collect::<Vec<_>>();
    json!({"items": items, "schema_version": state.schema_version})
}

pub(crate) fn identity_payload(state: &ApplicationState, filters: &Value) -> Value {
    let email = filters
        .get("email")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_lowercase());
    let id = filters.get("id").and_then(Value::as_str).map(str::trim);
    let user = state.user_accounts.iter().find(|user| {
        email.as_deref().is_some_and(|email| user.email == email)
            || id.is_some_and(|id| user.id == id)
    });
    json!({"user": user})
}
