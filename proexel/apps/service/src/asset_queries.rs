use std::collections::BTreeSet;

use proexel_application::ApplicationState;
use proexel_domain::PhotoOwnerType;
use serde_json::{json, Value};

use crate::query_support::{normalized_filter, operational_status_name, page_filter, text_filter};

pub(crate) fn categories_payload(state: &ApplicationState, filters: &Value) -> Value {
    let search = normalized_filter(filters, "search");
    let active = filters.get("active").and_then(Value::as_bool);
    let mut items = state
        .item_categories
        .iter()
        .filter(|category| {
            search.is_empty()
                || category.code_normalized.contains(&search)
                || category.name.to_uppercase().contains(&search)
        })
        .filter(|category| active.is_none_or(|active| category.active == active))
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.name.cmp(&right.name));
    let items = items
        .into_iter()
        .map(|category| {
            let step_ids = category
                .maintenance_guide
                .steps
                .iter()
                .map(|step| step.id.as_str())
                .collect::<BTreeSet<_>>();
            let mut value = serde_json::to_value(category).unwrap_or_else(|_| json!({}));
            value["guide_photos"] = json!(state
                .photos
                .iter()
                .filter(|photo| {
                    photo.owner_type == PhotoOwnerType::GuideStep
                        && step_ids.contains(photo.owner_id.as_str())
                })
                .collect::<Vec<_>>());
            value
        })
        .collect::<Vec<_>>();
    json!({"items": items, "total": items.len(), "schema_version": state.schema_version})
}

pub(crate) fn machines_payload(state: &ApplicationState, filters: &Value) -> Value {
    let id = text_filter(filters, "id");
    let search = normalized_filter(filters, "search");
    let zone = text_filter(filters, "zone");
    let status = text_filter(filters, "status");
    let page = page_filter(filters, "page", 1, usize::MAX);
    let page_size = page_filter(filters, "page_size", 25, 500);
    let include_removed = filters
        .get("include_removed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut machines = state
        .machines
        .iter()
        .filter(|machine| id.is_empty() || machine.id == id)
        .filter(|machine| {
            search.is_empty()
                || machine.code_normalized.contains(&search)
                || machine.name.to_uppercase().contains(&search)
                || machine.zone.to_uppercase().contains(&search)
        })
        .filter(|machine| zone.is_empty() || machine.zone == zone)
        .filter(|machine| status.is_empty() || operational_status_name(machine.status) == status)
        .collect::<Vec<_>>();
    machines.sort_by(|left, right| {
        left.zone
            .cmp(&right.zone)
            .then(left.code_normalized.cmp(&right.code_normalized))
    });
    let total = machines.len();
    let start = page.saturating_sub(1).saturating_mul(page_size).min(total);
    let items = machines
        .into_iter()
        .skip(start)
        .take(page_size)
        .map(|machine| {
            let mut value = serde_json::to_value(machine).unwrap_or_else(|_| json!({}));
            let machine_items = state
                .machine_items
                .iter()
                .filter(|item| item.machine_id == machine.id && (include_removed || item.active))
                .map(|item| {
                    let mut item_value = serde_json::to_value(item).unwrap_or_else(|_| json!({}));
                    item_value["category"] = json!(state
                        .item_categories
                        .iter()
                        .find(|category| category.id == item.category_id));
                    item_value["photos"] = json!(state
                        .photos
                        .iter()
                        .filter(|photo| photo.owner_type == PhotoOwnerType::MachineItem
                            && photo.owner_id == item.id)
                        .collect::<Vec<_>>());
                    item_value["replacement_history"] = json!(state
                        .machine_item_replacements
                        .iter()
                        .filter(|replacement| replacement.machine_item_id == item.id)
                        .map(|replacement| {
                            let mut replacement_value =
                                serde_json::to_value(replacement).unwrap_or_else(|_| json!({}));
                            replacement_value["photos"] = json!(state
                                .photos
                                .iter()
                                .filter(|photo| {
                                    photo.owner_type == PhotoOwnerType::Replacement
                                        && photo.owner_id == replacement.id
                                })
                                .collect::<Vec<_>>());
                            replacement_value
                        })
                        .collect::<Vec<_>>());
                    item_value
                })
                .collect::<Vec<_>>();
            value["items"] = json!(machine_items);
            value["photos"] = json!(state
                .photos
                .iter()
                .filter(|photo| photo.owner_type == PhotoOwnerType::Machine
                    && photo.owner_id == machine.id)
                .collect::<Vec<_>>());
            value
        })
        .collect::<Vec<_>>();
    let zones = state
        .machines
        .iter()
        .map(|machine| machine.zone.clone())
        .collect::<BTreeSet<_>>();
    json!({
        "items": items,
        "total": total,
        "page": page,
        "page_size": page_size,
        "facets": {"zones": zones},
        "schema_version": state.schema_version,
    })
}
