use std::collections::{BTreeMap, BTreeSet};

use proexel_application::ApplicationState;
use proexel_domain::{
    normalize_reference, normalize_tag, ComplexityLevel, CustomFieldDefinition, CustomFieldType,
    InstalledComponent, ItemCategory, Machine, MachineItem, MaintenanceGuide, OperationalStatus,
    ReplacementSpecification,
};

use crate::{
    migration_support::{clean, hash, insert_clean, machine_for_zone},
    report::inc,
    LegacyBundle, MigrationReport, LEGACY_CATEGORY_ID,
};

pub(crate) fn ensure_legacy_category(
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    if state
        .item_categories
        .iter()
        .any(|category| category.id == LEGACY_CATEGORY_ID)
    {
        return;
    }
    let definitions = [
        ("seat", "Seat", CustomFieldType::Text),
        ("dn", "Nominal diameter", CustomFieldType::Text),
        ("valve_type", "Type", CustomFieldType::Text),
        ("actuator", "Actuator", CustomFieldType::Text),
        ("manufactured_at", "Manufactured at", CustomFieldType::Date),
        (
            "last_kit_changed_at",
            "Last kit change",
            CustomFieldType::Date,
        ),
        (
            "last_maintenance_at",
            "Last maintenance",
            CustomFieldType::Date,
        ),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (key, label, field_type))| CustomFieldDefinition {
        id: format!("legacy-field-{key}"),
        key: key.to_string(),
        label: label.to_string(),
        field_type,
        required: false,
        unit: None,
        options: Vec::new(),
        minimum: None,
        maximum: None,
        order: index as u32,
    })
    .collect();
    state.item_categories.push(ItemCategory {
        id: LEGACY_CATEGORY_ID.to_string(),
        code: "VALVE".to_string(),
        code_normalized: "VALVE".to_string(),
        name: "Valve".to_string(),
        description: Some("Category imported from the previous PROEXEL model".to_string()),
        icon: Some("circle-dot".to_string()),
        default_complexity_level: ComplexityLevel::INTERMEDIATE,
        maintenance_guide: MaintenanceGuide::default(),
        custom_field_definitions: definitions,
        recommended_parts: Vec::new(),
        active: true,
        created_at_ms: now,
        updated_at_ms: now,
    });
    inc(report, "item_categories");
}

pub(crate) fn import_machines(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    let zones = bundle
        .valves
        .iter()
        .map(|item| item.zone.trim())
        .chain(bundle.orders.iter().map(|order| order.zone.trim()))
        .filter(|zone| !zone.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    for zone in zones {
        let zone_key = zone.to_uppercase();
        if state.machines.iter().any(|machine| {
            machine.zone.to_uppercase() == zone_key && machine.code.starts_with("LEGACY-")
        }) {
            continue;
        }
        let zone_hash = hash(&zone_key);
        state.machines.push(Machine {
            id: format!("legacy-machine-{zone_hash}"),
            code: format!("LEGACY-{}", &zone_hash[..8]),
            code_normalized: format!("LEGACY-{}", &zone_hash[..8]),
            name: format!("Legacy assets - {zone}"),
            description: Some("Machine generated while migrating legacy zone assets".to_string()),
            zone,
            location: None,
            manufacturer: None,
            model: None,
            serial_number: None,
            status: OperationalStatus::Unknown,
            main_photo_id: None,
            active: true,
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "machines");
    }
}

pub(crate) fn import_machine_items(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.valves {
        let tag = normalize_tag(&old.tag);
        let zone = old.zone.trim();
        if tag.is_empty() || zone.is_empty() {
            report
                .warnings
                .push(format!("asset skipped: empty TAG or zone ({})", old.tag));
            continue;
        }
        let Some(machine_id) = machine_for_zone(state, zone).map(|machine| machine.id.clone())
        else {
            report
                .warnings
                .push(format!("asset {tag} skipped: generated machine not found"));
            continue;
        };
        if state
            .machine_items
            .iter()
            .any(|item| item.machine_id == machine_id && item.code_normalized == tag)
        {
            continue;
        }
        let position = state
            .machine_items
            .iter()
            .filter(|item| item.machine_id == machine_id)
            .count() as u32;
        let mut custom_field_values = BTreeMap::new();
        insert_clean(&mut custom_field_values, "seat", &old.seat);
        insert_clean(&mut custom_field_values, "dn", &old.dn);
        insert_clean(&mut custom_field_values, "valve_type", &old.valve_type);
        insert_clean(&mut custom_field_values, "actuator", &old.actuator);
        insert_clean(
            &mut custom_field_values,
            "manufactured_at",
            &old.manufactured_at,
        );
        insert_clean(
            &mut custom_field_values,
            "last_kit_changed_at",
            &old.last_kit_changed_at,
        );
        insert_clean(
            &mut custom_field_values,
            "last_maintenance_at",
            &old.last_maintenance_at,
        );
        let item_hash = hash(&format!("{}|{}", zone.to_uppercase(), tag));
        let installed_component = if old.manufacturer.is_some() || old.serial.is_some() {
            Some(InstalledComponent {
                installation_id: format!("legacy-installation-{item_hash}"),
                manufacturer: clean(&old.manufacturer),
                model: None,
                part_number: None,
                serial_number: clean(&old.serial),
                installed_at: None,
                technical_specifications: BTreeMap::new(),
            })
        } else {
            None
        };
        let replacement_specification = ReplacementSpecification {
            part_number: old
                .kit_reference
                .as_deref()
                .map(normalize_reference)
                .filter(|value| !value.is_empty()),
            ..ReplacementSpecification::default()
        };
        state.machine_items.push(MachineItem {
            id: format!("legacy-machine-item-{item_hash}"),
            machine_id,
            category_id: LEGACY_CATEGORY_ID.to_string(),
            name: tag.clone(),
            code: tag.clone(),
            code_normalized: tag,
            complexity_level: ComplexityLevel::INTERMEDIATE,
            status: OperationalStatus::Unknown,
            position,
            location_description: None,
            custom_field_values,
            installed_component,
            replacement_specification,
            notes: None,
            active: true,
            removed_at_ms: None,
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "machine_items");
    }
}
