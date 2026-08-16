use std::collections::BTreeMap;

use proexel_application::ApplicationState;
use proexel_domain::{
    derive_machine_status, ItemCategory, ItemCategorySnapshot, Machine, MachineItem,
    MachineItemSnapshot, MachineSnapshot, RestockStatus, ServiceOrderPriority, ServiceOrderStatus,
};
use serde_json::{json, Value};

pub(crate) fn refresh_machine_statuses(state: &mut ApplicationState, now: u64) {
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

pub(crate) fn machine_for_zone<'a>(state: &'a ApplicationState, zone: &str) -> Option<&'a Machine> {
    let zone = zone.trim().to_uppercase();
    state
        .machines
        .iter()
        .find(|machine| machine.zone.to_uppercase() == zone && machine.code.starts_with("LEGACY-"))
}

pub(crate) fn find_item_by_tag<'a>(
    state: &'a ApplicationState,
    tag: &str,
) -> Option<&'a MachineItem> {
    state
        .machine_items
        .iter()
        .find(|item| item.code_normalized == tag)
}

pub(crate) fn category_snapshot(category: &ItemCategory) -> ItemCategorySnapshot {
    ItemCategorySnapshot {
        id: category.id.clone(),
        code: category.code.clone(),
        name: category.name.clone(),
        guide_version: category.maintenance_guide.version,
        maintenance_guide: category.maintenance_guide.clone(),
        guide_reference_photos: Vec::new(),
    }
}

pub(crate) fn item_snapshot(item: &MachineItem, category: &ItemCategory) -> MachineItemSnapshot {
    MachineItemSnapshot {
        id: item.id.clone(),
        machine_id: item.machine_id.clone(),
        category: category_snapshot(category),
        name: item.name.clone(),
        code: item.code.clone(),
        complexity_level: item.complexity_level,
        installed_component: item.installed_component.clone(),
    }
}

pub(crate) fn machine_snapshot(machine: &Machine) -> MachineSnapshot {
    MachineSnapshot {
        id: machine.id.clone(),
        code: machine.code.clone(),
        name: machine.name.clone(),
        zone: machine.zone.clone(),
        location: machine.location.clone(),
    }
}

pub(crate) fn insert_clean(
    values: &mut BTreeMap<String, Value>,
    key: &str,
    value: &Option<String>,
) {
    if let Some(value) = clean(value) {
        values.insert(key.to_string(), json!(value));
    }
}

pub(crate) fn clean(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn default_priority() -> String {
    "normal".into()
}

pub(crate) fn default_pending() -> String {
    "pending".into()
}

pub(crate) fn order_status(value: &str) -> ServiceOrderStatus {
    match value.trim().to_lowercase().as_str() {
        "andamento" | "en curso" | "en_cours" | "in_progress" => ServiceOrderStatus::InProgress,
        "concluida" | "concluída" | "completada" | "terminée" | "completed" => {
            ServiceOrderStatus::Completed
        }
        "cancelada" | "annulée" | "cancelled" | "canceled" => ServiceOrderStatus::Cancelled,
        _ => ServiceOrderStatus::Pending,
    }
}

pub(crate) fn order_priority(value: &str) -> ServiceOrderPriority {
    match value.trim().to_lowercase().as_str() {
        "low" | "baixa" | "baja" | "faible" => ServiceOrderPriority::Low,
        "high" | "alta" | "haute" => ServiceOrderPriority::High,
        "urgent" | "urgente" => ServiceOrderPriority::Urgent,
        _ => ServiceOrderPriority::Normal,
    }
}

pub(crate) fn restock_status(value: &str) -> RestockStatus {
    match value.trim().to_lowercase().as_str() {
        "aprovada" | "aprobada" | "approuvée" | "approved" => RestockStatus::Approved,
        "rejeitada" | "rechazada" | "rejetée" | "rejected" => RestockStatus::Rejected,
        _ => RestockStatus::Pending,
    }
}

pub(crate) fn date_to_ms(value: &str) -> Option<u64> {
    let date = value.trim().get(..10)?;
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    let days = days_from_civil(year, month, day)?;
    (days >= 0).then_some(days as u64 * 86_400_000)
}

pub(crate) fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
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

pub(crate) fn hash(value: &str) -> String {
    format!("{:016x}", hash_bytes(value.as_bytes()))
}

pub(crate) fn hash_bytes(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}
