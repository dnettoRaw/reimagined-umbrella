use std::collections::BTreeMap;

use proexel_domain::{
    ComplexityLevel, ItemCategory, ItemCategorySnapshot, Machine, MachineItem, MachineItemSnapshot,
    MachineSnapshot,
};
use serde_json::{json, Value};

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

pub(crate) fn insert_optional(
    values: &mut BTreeMap<String, Value>,
    key: &str,
    value: Option<String>,
) {
    if let Some(value) = clean(value) {
        values.insert(key.to_string(), json!(value));
    }
}

pub(crate) fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn date_to_ms(value: &str) -> Option<u64> {
    let date = value.trim().get(..10)?;
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let shifted_month = month as i32 + if month > 2 { -3 } else { 9 };
    let doy = (153 * shifted_month + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = (era * 146_097 + doe - 719_468) as i64;
    (days >= 0).then_some(days as u64 * 86_400_000)
}

pub(crate) fn hash(value: &str) -> String {
    format!(
        "{:016x}",
        value
            .as_bytes()
            .iter()
            .fold(0xcbf29ce484222325_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
            })
    )
}

pub(crate) fn default_repair_level(role: &str) -> ComplexityLevel {
    if matches!(role, "admin" | "chefe") {
        ComplexityLevel::EXPERT
    } else {
        ComplexityLevel::INTERMEDIATE
    }
}
