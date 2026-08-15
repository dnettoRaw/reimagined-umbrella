use std::collections::{BTreeMap, BTreeSet};

use proexel_domain::{
    derive_machine_status, normalize_identifier, ComplexityLevel, InstalledComponent, MachineItem,
    MachineItemReplacement, OperationalStatus, ReplacementSpecification, ServiceOrderStatus,
    ServiceOrderTaskStatus,
};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    state::{
        clean_optional, domain_action, json_string, parse_data, require_permission, require_text,
        Action, CommandPayload,
    },
    ApplicationState,
};

use crate::asset_audit::SupplementalAudit;
use crate::asset_validation::validate_custom_values;

impl ApplicationState {
    pub(crate) fn add_machine_item(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "machine_item.manage")?;
        let input: MachineItemInput = parse_data(payload)?;
        let machine = self
            .machines
            .iter()
            .find(|machine| machine.id == input.machine_id && machine.active)
            .ok_or_else(|| "machine_not_found".to_string())?;
        let category = self
            .item_categories
            .iter()
            .find(|category| category.id == input.category_id)
            .ok_or_else(|| "category_not_found".to_string())?;
        if !category.active {
            return Err("category_inactive".to_string());
        }
        require_text(&input.name, "machine_item_name_required")?;
        let code = normalize_identifier(&input.code);
        require_text(&code, "machine_item_code_required")?;
        if self.machine_items.iter().any(|item| {
            item.machine_id == input.machine_id && item.active && item.code_normalized == code
        }) {
            return Err("machine_item_code_already_exists".to_string());
        }
        validate_custom_values(category, &input.custom_field_values)?;
        let complexity = input
            .complexity_level
            .unwrap_or(category.default_complexity_level);
        let position = self
            .machine_items
            .iter()
            .filter(|item| item.machine_id == machine.id && item.active)
            .map(|item| item.position)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let id = format!("machine-item-{command_id}");
        let item = MachineItem {
            id: id.clone(),
            machine_id: input.machine_id.clone(),
            category_id: input.category_id,
            name: input.name.trim().to_string(),
            code: code.clone(),
            code_normalized: code,
            complexity_level: complexity,
            status: input.status.unwrap_or_default(),
            position,
            location_description: clean_optional(input.location_description),
            custom_field_values: input.custom_field_values,
            installed_component: input
                .installed_component
                .map(|component| component.into_installed(format!("installation-{command_id}"))),
            replacement_specification: input.replacement_specification.unwrap_or_default(),
            notes: clean_optional(input.notes),
            active: true,
            removed_at_ms: None,
            created_at_ms: now,
            updated_at_ms: now,
        };
        let after = json_string(&item);
        self.machine_items.push(item);
        self.refresh_machine_status(&input.machine_id, now);
        Ok(domain_action(
            "machine_item.manage",
            "machine_item.created",
            "machine_item",
            id,
            None,
            after,
            "Machine item created",
        ))
    }

    pub(crate) fn update_machine_item(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "machine_item.manage")?;
        let input: UpdateMachineItem = parse_data(payload)?;
        let item_index = self
            .machine_items
            .iter()
            .position(|item| item.id == input.id && item.active)
            .ok_or_else(|| "machine_item_not_found".to_string())?;
        let machine_id = self.machine_items[item_index].machine_id.clone();
        let category = self
            .item_categories
            .iter()
            .find(|category| category.id == input.item.category_id)
            .ok_or_else(|| "category_not_found".to_string())?;
        if !category.active {
            return Err("category_inactive".to_string());
        }
        require_text(&input.item.name, "machine_item_name_required")?;
        let code = normalize_identifier(&input.item.code);
        require_text(&code, "machine_item_code_required")?;
        if self.machine_items.iter().any(|item| {
            item.id != input.id
                && item.machine_id == machine_id
                && item.active
                && item.code_normalized == code
        }) {
            return Err("machine_item_code_already_exists".to_string());
        }
        validate_custom_values(category, &input.item.custom_field_values)?;
        let before = json_string(&self.machine_items[item_index]);
        let previous_status = self.machine_items[item_index].status;
        let item = &mut self.machine_items[item_index];
        item.category_id = input.item.category_id;
        item.name = input.item.name.trim().to_string();
        item.code = code.clone();
        item.code_normalized = code;
        item.complexity_level = input.item.complexity_level;
        item.status = input.item.status;
        item.location_description = clean_optional(input.item.location_description);
        item.custom_field_values = input.item.custom_field_values;
        item.replacement_specification = input.item.replacement_specification;
        item.notes = clean_optional(input.item.notes);
        item.updated_at_ms = now;
        let status_after = item.status;
        let after = json_string(item);
        if previous_status != status_after {
            self.push_supplemental_audit(
                payload,
                now,
                SupplementalAudit {
                    operation: "item.status_changed",
                    aggregate: "machine_item",
                    aggregate_id: &input.id,
                    before: json_string(&previous_status),
                    after: json_string(&status_after),
                    description: "Machine item status changed",
                },
            );
        }
        self.refresh_machine_status(&machine_id, now);
        Ok(domain_action(
            "machine_item.manage",
            "machine_item.updated",
            "machine_item",
            input.id,
            before,
            after,
            "Machine item updated",
        ))
    }

    pub(crate) fn reorder_machine_items(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "machine_item.manage")?;
        let input: ReorderMachineItems = parse_data(payload)?;
        let active_ids = self
            .machine_items
            .iter()
            .filter(|item| item.machine_id == input.machine_id && item.active)
            .map(|item| item.id.clone())
            .collect::<BTreeSet<_>>();
        let requested = input.item_ids.iter().cloned().collect::<BTreeSet<_>>();
        if active_ids != requested || requested.len() != input.item_ids.len() {
            return Err("machine_item_reorder_invalid".to_string());
        }
        let before = json_string(
            &self
                .machine_items
                .iter()
                .filter(|item| item.machine_id == input.machine_id && item.active)
                .map(|item| (&item.id, item.position))
                .collect::<Vec<_>>(),
        );
        for (position, id) in input.item_ids.iter().enumerate() {
            if let Some(item) = self.machine_items.iter_mut().find(|item| item.id == *id) {
                item.position = position as u32 + 1;
                item.updated_at_ms = now;
            }
        }
        let after = json_string(&input.item_ids);
        Ok(domain_action(
            "machine_item.manage",
            "machine_item.updated",
            "machine",
            input.machine_id,
            before,
            after,
            "Machine items reordered",
        ))
    }

    pub(crate) fn remove_machine_item(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "machine_item.manage")?;
        let input: RemoveMachineItem = parse_data(payload)?;
        if self.service_orders.iter().any(|order| {
            !matches!(
                order.status,
                ServiceOrderStatus::Completed | ServiceOrderStatus::Cancelled
            ) && order.tasks.iter().any(|task| {
                task.machine_item_id == input.id && task.status != ServiceOrderTaskStatus::Completed
            })
        }) {
            return Err("machine_item_has_open_tasks".to_string());
        }
        let item = self
            .machine_items
            .iter_mut()
            .find(|item| item.id == input.id && item.active)
            .ok_or_else(|| "machine_item_not_found".to_string())?;
        let before = json_string(item);
        let machine_id = item.machine_id.clone();
        item.active = false;
        item.status = OperationalStatus::Disabled;
        item.removed_at_ms = Some(now);
        item.updated_at_ms = now;
        let after = json_string(item);
        self.refresh_machine_status(&machine_id, now);
        Ok(domain_action(
            "machine_item.manage",
            "machine_item.removed",
            "machine_item",
            input.id,
            before,
            after,
            "Machine item removed",
        ))
    }

    pub(crate) fn replace_machine_item(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "machine_item.manage")?;
        let input: ReplaceMachineItem = parse_data(payload)?;
        require_text(&input.reason, "replacement_reason_required")?;
        let item = self
            .machine_items
            .iter_mut()
            .find(|item| item.id == input.id && item.active)
            .ok_or_else(|| "machine_item_not_found".to_string())?;
        let before = json_string(item);
        let previous = item.installed_component.clone();
        let current = input
            .installed_component
            .into_installed(format!("installation-{command_id}"));
        item.installed_component = Some(current.clone());
        item.updated_at_ms = now;
        let after = json_string(item);
        self.machine_item_replacements.push(MachineItemReplacement {
            id: format!("replacement-{command_id}"),
            machine_item_id: input.id.clone(),
            previous,
            current,
            reason: input.reason.trim().to_string(),
            replaced_by: payload.actor.name.clone(),
            replaced_at_ms: now,
        });
        Ok(domain_action(
            "machine_item.manage",
            "machine_item.replaced",
            "machine_item",
            input.id,
            before,
            after,
            "Machine item physical unit replaced",
        ))
    }

    pub(crate) fn refresh_machine_status(&mut self, machine_id: &str, now: u64) {
        let statuses = self
            .machine_items
            .iter()
            .filter(|item| item.machine_id == machine_id && item.active)
            .map(|item| &item.status)
            .collect::<Vec<_>>();
        let derived = derive_machine_status(statuses);
        if let Some(machine) = self
            .machines
            .iter_mut()
            .find(|machine| machine.id == machine_id)
        {
            machine.status = derived;
            machine.updated_at_ms = now;
        }
    }
}

#[derive(Deserialize)]
struct InstalledComponentInput {
    manufacturer: Option<String>,
    model: Option<String>,
    part_number: Option<String>,
    serial_number: Option<String>,
    installed_at: Option<String>,
    #[serde(default)]
    technical_specifications: BTreeMap<String, Value>,
}

impl InstalledComponentInput {
    fn into_installed(self, installation_id: String) -> InstalledComponent {
        InstalledComponent {
            installation_id,
            manufacturer: clean_optional(self.manufacturer),
            model: clean_optional(self.model),
            part_number: clean_optional(self.part_number),
            serial_number: clean_optional(self.serial_number),
            installed_at: clean_optional(self.installed_at),
            technical_specifications: self.technical_specifications,
        }
    }
}

#[derive(Deserialize)]
struct MachineItemInput {
    machine_id: String,
    category_id: String,
    name: String,
    code: String,
    complexity_level: Option<ComplexityLevel>,
    status: Option<OperationalStatus>,
    location_description: Option<String>,
    #[serde(default)]
    custom_field_values: BTreeMap<String, Value>,
    installed_component: Option<InstalledComponentInput>,
    replacement_specification: Option<ReplacementSpecification>,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct UpdateMachineItemData {
    category_id: String,
    name: String,
    code: String,
    complexity_level: ComplexityLevel,
    status: OperationalStatus,
    location_description: Option<String>,
    #[serde(default)]
    custom_field_values: BTreeMap<String, Value>,
    #[serde(default)]
    replacement_specification: ReplacementSpecification,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct UpdateMachineItem {
    id: String,
    #[serde(flatten)]
    item: UpdateMachineItemData,
}

#[derive(Deserialize)]
struct ReorderMachineItems {
    machine_id: String,
    item_ids: Vec<String>,
}

#[derive(Deserialize)]
struct RemoveMachineItem {
    id: String,
}

#[derive(Deserialize)]
struct ReplaceMachineItem {
    id: String,
    reason: String,
    installed_component: InstalledComponentInput,
}
