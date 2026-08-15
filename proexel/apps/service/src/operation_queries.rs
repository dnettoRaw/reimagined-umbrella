use std::collections::BTreeSet;

use proexel_application::ApplicationState;
use proexel_domain::{PhotoOwnerType, ServiceOrderTaskStatus};
use serde_json::{json, Value};

use crate::query_support::{order_status_name, page_filter, text_filter};

pub(crate) fn orders_payload(state: &ApplicationState, filters: &Value) -> Value {
    let id = text_filter(filters, "id");
    let machine_id = text_filter(filters, "machine_id");
    let status = text_filter(filters, "status");
    let operator_id = text_filter(filters, "operator_id");
    let mut orders = state
        .service_orders
        .iter()
        .filter(|order| id.is_empty() || order.id == id)
        .filter(|order| machine_id.is_empty() || order.machine_id == machine_id)
        .filter(|order| status.is_empty() || order_status_name(order.status) == status)
        .filter(|order| {
            operator_id.is_empty()
                || order
                    .tasks
                    .iter()
                    .any(|task| task.assigned_operator_id.as_deref() == Some(operator_id.as_str()))
        })
        .collect::<Vec<_>>();
    orders.sort_by_key(|order| std::cmp::Reverse(order.created_at_ms));
    let items = orders
        .into_iter()
        .map(|order| {
            let mut value = serde_json::to_value(order).unwrap_or_else(|_| json!({}));
            value["maximum_complexity_level"] = json!(order
                .tasks
                .iter()
                .map(|task| task.complexity_snapshot.get())
                .max()
                .unwrap_or(1));
            value["completed_tasks"] = json!(order
                .tasks
                .iter()
                .filter(|task| task.status == ServiceOrderTaskStatus::Completed)
                .count());
            value
        })
        .collect::<Vec<_>>();
    json!({"total": items.len(), "items": items, "schema_version": state.schema_version})
}

pub(crate) fn inspections_payload(state: &ApplicationState, filters: &Value) -> Value {
    let id = text_filter(filters, "id");
    let order_id = text_filter(filters, "service_order_id");
    let machine_id = text_filter(filters, "machine_id");
    let machine_item_id = text_filter(filters, "machine_item_id");
    let operator_id = text_filter(filters, "operator_id");
    let mut items = state
        .inspections
        .iter()
        .filter(|inspection| id.is_empty() || inspection.id == id)
        .filter(|inspection| {
            order_id.is_empty() || inspection.service_order_id.as_deref() == Some(order_id.as_str())
        })
        .filter(|inspection| machine_id.is_empty() || inspection.machine_id == machine_id)
        .filter(|inspection| {
            machine_item_id.is_empty() || inspection.machine_item_id == machine_item_id
        })
        .filter(|inspection| operator_id.is_empty() || inspection.operator_id == operator_id)
        .collect::<Vec<_>>();
    items.sort_by_key(|inspection| std::cmp::Reverse(inspection.started_at_ms));
    let items = items
        .into_iter()
        .map(|inspection| {
            let mut value = serde_json::to_value(inspection).unwrap_or_else(|_| json!({}));
            value["photos"] = json!(state
                .photos
                .iter()
                .filter(|photo| {
                    photo.owner_type == PhotoOwnerType::Inspection
                        && photo.owner_id == inspection.id
                })
                .collect::<Vec<_>>());
            value
        })
        .collect::<Vec<_>>();
    json!({"total": items.len(), "items": items, "schema_version": state.schema_version})
}

pub(crate) fn audit_payload(state: &ApplicationState, filters: &Value) -> Value {
    let search = text_filter(filters, "search").to_lowercase();
    let operation = text_filter(filters, "operation");
    let actor = text_filter(filters, "actor");
    let aggregate = text_filter(filters, "aggregate");
    let from_ms = filters.get("from_ms").and_then(Value::as_u64).unwrap_or(0);
    let to_ms = filters
        .get("to_ms")
        .and_then(Value::as_u64)
        .unwrap_or(u64::MAX);
    let page = page_filter(filters, "page", 1, usize::MAX);
    let page_size = page_filter(filters, "page_size", 50, 200);
    let events = state
        .audit_events
        .iter()
        .rev()
        .filter(|event| {
            search.is_empty()
                || event.actor.to_lowercase().contains(&search)
                || event.operation.to_lowercase().contains(&search)
                || event.aggregate_id.to_lowercase().contains(&search)
                || event
                    .description
                    .as_deref()
                    .is_some_and(|value| value.to_lowercase().contains(&search))
        })
        .filter(|event| operation.is_empty() || event.operation == operation)
        .filter(|event| actor.is_empty() || event.actor == actor)
        .filter(|event| aggregate.is_empty() || event.aggregate == aggregate)
        .filter(|event| event.created_at_ms >= from_ms && event.created_at_ms <= to_ms)
        .collect::<Vec<_>>();
    let total = events.len();
    let start = page.saturating_sub(1).saturating_mul(page_size).min(total);
    let items = events
        .into_iter()
        .skip(start)
        .take(page_size)
        .collect::<Vec<_>>();
    let operations = state
        .audit_events
        .iter()
        .map(|event| event.operation.clone())
        .collect::<BTreeSet<_>>();
    let actors = state
        .audit_events
        .iter()
        .map(|event| event.actor.clone())
        .collect::<BTreeSet<_>>();
    let aggregates = state
        .audit_events
        .iter()
        .map(|event| event.aggregate.clone())
        .collect::<BTreeSet<_>>();
    json!({
        "items": items,
        "schema_version": state.schema_version,
        "total": total,
        "page": page,
        "page_size": page_size,
        "operations": operations,
        "actors": actors,
        "aggregates": aggregates,
    })
}
