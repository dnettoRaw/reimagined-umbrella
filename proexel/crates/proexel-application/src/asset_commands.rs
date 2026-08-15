use std::collections::{BTreeMap, BTreeSet};

use proexel_domain::{
    can_execute_complexity, derive_machine_status, normalize_identifier, ComplexityLevel,
    CustomFieldDefinition, CustomFieldType, GuideStepType, InspectionFinding, InspectionStatus,
    InspectionStepResult, InstalledComponent, ItemCategory, ItemCategorySnapshot, ItemInspection,
    Machine, MachineItem, MachineItemReplacement, MachineItemSnapshot, MachineSnapshot,
    MaintenanceGuide, OperationalStatus, PhotoAsset, PhotoOwnerType, PhotoPurpose, RecommendedPart,
    ReplacementSpecification, ServiceOrder, ServiceOrderPriority, ServiceOrderStatus,
    ServiceOrderTask, ServiceOrderTaskStatus, UserAccount,
};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    state::{
        clean_optional, domain_action, json_string, parse_data, require_permission, require_text,
        Action, CommandPayload,
    },
    ApplicationState, Role,
};

impl ApplicationState {
    pub(crate) fn create_item_category(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "item_category.manage")?;
        let input: CategoryInput = parse_data(payload)?;
        let code = normalize_identifier(&input.code);
        require_text(&code, "category_code_required")?;
        require_text(&input.name, "category_name_required")?;
        if self
            .item_categories
            .iter()
            .any(|category| category.code_normalized == code)
        {
            return Err("category_code_already_exists".to_string());
        }
        let mut guide = input.maintenance_guide;
        guide.version = 1;
        validate_category_definition(&input.custom_field_definitions, &guide)?;
        let id = format!("category-{command_id}");
        let category = ItemCategory {
            id: id.clone(),
            code: code.clone(),
            code_normalized: code,
            name: input.name.trim().to_string(),
            description: clean_optional(input.description),
            icon: clean_optional(input.icon),
            default_complexity_level: input.default_complexity_level,
            maintenance_guide: normalized_guide(guide),
            custom_field_definitions: normalized_fields(input.custom_field_definitions),
            recommended_parts: input.recommended_parts,
            active: input.active,
            created_at_ms: now,
            updated_at_ms: now,
        };
        let after = json_string(&category);
        self.item_categories.push(category);
        Ok(domain_action(
            "item_category.manage",
            "item_category.created",
            "item_category",
            id,
            None,
            after,
            "Item category created",
        ))
    }

    pub(crate) fn update_item_category(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "item_category.manage")?;
        let input: UpdateCategory = parse_data(payload)?;
        let code = normalize_identifier(&input.category.code);
        require_text(&code, "category_code_required")?;
        require_text(&input.category.name, "category_name_required")?;
        let index = self
            .item_categories
            .iter()
            .position(|category| category.id == input.id)
            .ok_or_else(|| "category_not_found".to_string())?;
        if self
            .item_categories
            .iter()
            .any(|category| category.id != input.id && category.code_normalized == code)
        {
            return Err("category_code_already_exists".to_string());
        }
        validate_category_definition(
            &input.category.custom_field_definitions,
            &input.category.maintenance_guide,
        )?;
        let before = json_string(&self.item_categories[index]);
        let previous_guide = self.item_categories[index].maintenance_guide.clone();
        let guide_changed = previous_guide.steps != input.category.maintenance_guide.steps;
        let version = if guide_changed {
            previous_guide.version.saturating_add(1)
        } else {
            previous_guide.version
        };
        let category = &mut self.item_categories[index];
        category.code = code.clone();
        category.code_normalized = code;
        category.name = input.category.name.trim().to_string();
        category.description = clean_optional(input.category.description);
        category.icon = clean_optional(input.category.icon);
        category.default_complexity_level = input.category.default_complexity_level;
        category.maintenance_guide = normalized_guide(MaintenanceGuide {
            version,
            steps: input.category.maintenance_guide.steps,
        });
        category.custom_field_definitions =
            normalized_fields(input.category.custom_field_definitions);
        category.recommended_parts = input.category.recommended_parts;
        category.active = input.category.active;
        category.updated_at_ms = now;
        let guide_after = json_string(&category.maintenance_guide);
        let after = json_string(category);
        if guide_changed {
            self.push_supplemental_audit(
                payload,
                now,
                "maintenance_guide.updated",
                "item_category",
                &input.id,
                json_string(&previous_guide),
                guide_after,
                "Maintenance guide updated",
            );
        }
        Ok(domain_action(
            "item_category.manage",
            "item_category.updated",
            "item_category",
            input.id,
            before,
            after,
            "Item category updated",
        ))
    }

    pub(crate) fn create_machine(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "machine.create")?;
        let input: MachineInput = parse_data(payload)?;
        let code = normalize_identifier(&input.code);
        require_text(&code, "machine_code_required")?;
        require_text(&input.name, "machine_name_required")?;
        require_text(&input.zone, "zone_required")?;
        if self
            .machines
            .iter()
            .any(|machine| machine.code_normalized == code)
        {
            return Err("machine_code_already_exists".to_string());
        }
        let id = format!("machine-{command_id}");
        let machine = Machine {
            id: id.clone(),
            code: code.clone(),
            code_normalized: code,
            name: input.name.trim().to_string(),
            description: clean_optional(input.description),
            zone: input.zone.trim().to_string(),
            location: clean_optional(input.location),
            manufacturer: clean_optional(input.manufacturer),
            model: clean_optional(input.model),
            serial_number: clean_optional(input.serial_number),
            status: OperationalStatus::Unknown,
            main_photo_id: None,
            active: input.active,
            created_at_ms: now,
            updated_at_ms: now,
        };
        let after = json_string(&machine);
        self.machines.push(machine);
        Ok(domain_action(
            "machine.create",
            "machine.created",
            "machine",
            id,
            None,
            after,
            "Machine created",
        ))
    }

    pub(crate) fn update_machine(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "machine.update")?;
        let input: UpdateMachine = parse_data(payload)?;
        let code = normalize_identifier(&input.machine.code);
        require_text(&code, "machine_code_required")?;
        require_text(&input.machine.name, "machine_name_required")?;
        require_text(&input.machine.zone, "zone_required")?;
        let index = self
            .machines
            .iter()
            .position(|machine| machine.id == input.id)
            .ok_or_else(|| "machine_not_found".to_string())?;
        if self
            .machines
            .iter()
            .any(|machine| machine.id != input.id && machine.code_normalized == code)
        {
            return Err("machine_code_already_exists".to_string());
        }
        let before = json_string(&self.machines[index]);
        let machine = &mut self.machines[index];
        machine.code = code.clone();
        machine.code_normalized = code;
        machine.name = input.machine.name.trim().to_string();
        machine.description = clean_optional(input.machine.description);
        machine.zone = input.machine.zone.trim().to_string();
        machine.location = clean_optional(input.machine.location);
        machine.manufacturer = clean_optional(input.machine.manufacturer);
        machine.model = clean_optional(input.machine.model);
        machine.serial_number = clean_optional(input.machine.serial_number);
        machine.active = input.machine.active;
        machine.updated_at_ms = now;
        let after = json_string(machine);
        Ok(domain_action(
            "machine.update",
            "machine.updated",
            "machine",
            input.id,
            before,
            after,
            "Machine updated",
        ))
    }

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
                "item.status_changed",
                "machine_item",
                &input.id,
                json_string(&previous_status),
                json_string(&status_after),
                "Machine item status changed",
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

    pub(crate) fn add_photo(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        let input: AddPhoto = parse_data(payload)?;
        let permission = self.photo_permission(&input.owner_type, &input.owner_id, payload)?;
        require_permission(payload.actor.role, permission)?;
        require_text(&input.blob_ref, "photo_ref_required")?;
        if self
            .photos
            .iter()
            .any(|photo| photo.blob_ref == input.blob_ref)
        {
            return Err("photo_already_exists".to_string());
        }
        let id = format!("photo-{command_id}");
        let photo = PhotoAsset {
            id: id.clone(),
            owner_type: input.owner_type,
            owner_id: input.owner_id,
            purpose: input.purpose,
            blob_ref: input.blob_ref.trim().to_string(),
            description: clean_optional(input.description),
            created_by: payload.actor.name.clone(),
            created_at_ms: now,
        };
        let after = json_string(&photo);
        if photo.owner_type == PhotoOwnerType::Machine && photo.purpose == PhotoPurpose::Main {
            if let Some(machine) = self
                .machines
                .iter_mut()
                .find(|machine| machine.id == photo.owner_id)
            {
                machine.main_photo_id = Some(id.clone());
                machine.updated_at_ms = now;
            }
        }
        if photo.owner_type == PhotoOwnerType::GuideStep {
            for category in &mut self.item_categories {
                if let Some(step) = category
                    .maintenance_guide
                    .steps
                    .iter_mut()
                    .find(|step| step.id == photo.owner_id)
                {
                    if !step.reference_photo_ids.contains(&id) {
                        step.reference_photo_ids.push(id.clone());
                        category.maintenance_guide.version =
                            category.maintenance_guide.version.saturating_add(1);
                    }
                    category.updated_at_ms = now;
                    break;
                }
            }
        }
        self.photos.push(photo);
        Ok(domain_action(
            permission,
            "photo.added",
            "photo",
            id,
            None,
            after,
            "Photo added",
        ))
    }

    pub(crate) fn delete_photo(&mut self, payload: &CommandPayload) -> Result<Action, String> {
        let input: DeletePhoto = parse_data(payload)?;
        let index = self
            .photos
            .iter()
            .position(|photo| photo.id == input.id)
            .ok_or_else(|| "photo_not_found".to_string())?;
        let photo = self.photos[index].clone();
        if input.blob_ref != photo.blob_ref {
            return Err("photo_ref_mismatch".to_string());
        }
        let permission = self.photo_permission(&photo.owner_type, &photo.owner_id, payload)?;
        require_permission(payload.actor.role, permission)?;
        if self.service_orders.iter().any(|order| {
            order.tasks.iter().any(|task| {
                task.item_snapshot
                    .category
                    .guide_reference_photos
                    .iter()
                    .any(|snapshot| snapshot.id == photo.id)
            })
        }) {
            return Err("photo_in_use_by_service_order".to_string());
        }
        self.photos.remove(index);
        for machine in &mut self.machines {
            if machine.main_photo_id.as_deref() == Some(&photo.id) {
                machine.main_photo_id = None;
            }
        }
        for category in &mut self.item_categories {
            let mut changed = false;
            for step in &mut category.maintenance_guide.steps {
                let previous_len = step.reference_photo_ids.len();
                step.reference_photo_ids.retain(|id| id != &photo.id);
                changed |= step.reference_photo_ids.len() != previous_len;
            }
            if changed {
                category.maintenance_guide.version =
                    category.maintenance_guide.version.saturating_add(1);
            }
        }
        Ok(domain_action(
            permission,
            "photo.removed",
            "photo",
            photo.id.clone(),
            json_string(&photo),
            None,
            "Photo removed",
        ))
    }

    pub(crate) fn create_service_order(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "order.create")?;
        let input: CreateServiceOrder = parse_data(payload)?;
        require_text(&input.description, "description_required")?;
        let machine = self
            .machines
            .iter()
            .find(|machine| machine.id == input.machine_id && machine.active)
            .ok_or_else(|| "machine_not_found".to_string())?
            .clone();
        let selected_ids = if input.all_items {
            self.machine_items
                .iter()
                .filter(|item| item.machine_id == machine.id && item.active)
                .map(|item| item.id.clone())
                .collect::<Vec<_>>()
        } else {
            input.item_ids
        };
        let unique = selected_ids.iter().cloned().collect::<BTreeSet<_>>();
        if selected_ids.is_empty() || unique.len() != selected_ids.len() {
            return Err("service_order_items_required".to_string());
        }
        let mut tasks = Vec::with_capacity(selected_ids.len());
        for (index, item_id) in selected_ids.iter().enumerate() {
            let item = self
                .machine_items
                .iter()
                .find(|item| item.id == *item_id && item.machine_id == machine.id && item.active)
                .ok_or_else(|| "machine_item_machine_mismatch".to_string())?;
            let category = self
                .item_categories
                .iter()
                .find(|category| category.id == item.category_id)
                .ok_or_else(|| "category_not_found".to_string())?;
            if let Some(operator_id) = input.assigned_operator_id.as_deref() {
                self.ensure_operator_level(operator_id, item.complexity_level)?;
            }
            tasks.push(ServiceOrderTask {
                id: format!("task-{command_id}-{index}"),
                machine_item_id: item.id.clone(),
                item_snapshot: item_snapshot(item, category, &self.photos),
                complexity_snapshot: item.complexity_level,
                assigned_operator_id: input.assigned_operator_id.clone(),
                status: ServiceOrderTaskStatus::Pending,
                started_at_ms: None,
                completed_at_ms: None,
                inspection_id: None,
            });
        }
        let id = format!("order-{command_id}");
        let order = ServiceOrder {
            id: id.clone(),
            machine_id: machine.id.clone(),
            machine_snapshot: MachineSnapshot {
                id: machine.id,
                code: machine.code,
                name: machine.name,
                zone: machine.zone,
            },
            description: input.description.trim().to_string(),
            priority: input.priority,
            status: ServiceOrderStatus::Pending,
            created_by: payload.actor.name.clone(),
            scheduled_for: clean_optional(input.scheduled_for),
            tasks,
            created_at_ms: now,
            started_at_ms: None,
            completed_at_ms: None,
            updated_at_ms: now,
        };
        let after = json_string(&order);
        self.service_orders.push(order);
        Ok(domain_action(
            "order.create",
            "service_order.created",
            "service_order",
            id,
            None,
            after,
            "Service order created with immutable item snapshots",
        ))
    }

    pub(crate) fn start_service_order(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "inspection.execute")?;
        let input: StartServiceOrder = parse_data(payload)?;
        let order_index = self
            .service_orders
            .iter()
            .position(|order| order.id == input.id)
            .ok_or_else(|| "order_not_found".to_string())?;
        let maximum = self.service_orders[order_index]
            .tasks
            .iter()
            .map(|task| task.complexity_snapshot)
            .max()
            .ok_or_else(|| "service_order_items_required".to_string())?;
        let operator_id = input.operator_id.as_deref().unwrap_or(&payload.actor.id);
        self.ensure_actor_may_act_as(payload, operator_id)?;
        self.ensure_operator_level(operator_id, maximum)?;
        let order = &mut self.service_orders[order_index];
        if order.status != ServiceOrderStatus::Pending {
            return Err("service_order_not_pending".to_string());
        }
        let before = json_string(order);
        order.status = ServiceOrderStatus::InProgress;
        order.started_at_ms = Some(now);
        order.updated_at_ms = now;
        let after = json_string(order);
        Ok(domain_action(
            "inspection.execute",
            "service_order.started",
            "service_order",
            input.id,
            before,
            after,
            "Service order started",
        ))
    }

    pub(crate) fn assign_order_task(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "order.manage")?;
        let input: AssignOrderTask = parse_data(payload)?;
        let (order_index, task_index) = self.find_task(&input.order_id, &input.task_id)?;
        let required = self.service_orders[order_index].tasks[task_index].complexity_snapshot;
        self.ensure_operator_level(&input.operator_id, required)?;
        let task = &mut self.service_orders[order_index].tasks[task_index];
        if task.status != ServiceOrderTaskStatus::Pending {
            return Err("service_order_task_already_started".to_string());
        }
        let before = json_string(task);
        task.assigned_operator_id = Some(input.operator_id);
        let after = json_string(task);
        self.service_orders[order_index].updated_at_ms = now;
        Ok(domain_action(
            "order.manage",
            "service_order.updated",
            "service_order_task",
            input.task_id,
            before,
            after,
            "Service order task assigned",
        ))
    }

    pub(crate) fn delete_service_order(
        &mut self,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "order.delete")?;
        let input: DeletePhoto = parse_data(payload)?;
        let index = self
            .service_orders
            .iter()
            .position(|order| order.id == input.id)
            .ok_or_else(|| "order_not_found".to_string())?;
        if self.service_orders[index].status != ServiceOrderStatus::Pending {
            return Err("started_order_cannot_be_deleted".to_string());
        }
        let order = self.service_orders.remove(index);
        Ok(domain_action(
            "order.delete",
            "service_order.deleted",
            "service_order",
            order.id.clone(),
            json_string(&order),
            None,
            "Service order deleted",
        ))
    }

    pub(crate) fn complete_service_order(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        let input: CompleteServiceOrder = parse_data(payload)?;
        let order_index = self
            .service_orders
            .iter()
            .position(|order| order.id == input.id)
            .ok_or_else(|| "order_not_found".to_string())?;
        let operator_id = input.operator_id.as_deref().unwrap_or(&payload.actor.id);
        self.ensure_actor_may_act_as(payload, operator_id)?;
        require_permission(payload.actor.role, "inspection.execute")?;
        let order = &mut self.service_orders[order_index];
        if order.status != ServiceOrderStatus::InProgress {
            return Err("service_order_not_in_progress".to_string());
        }
        if order
            .tasks
            .iter()
            .any(|task| task.status != ServiceOrderTaskStatus::Completed)
        {
            return Err("service_order_has_pending_tasks".to_string());
        }
        let before = json_string(order);
        order.status = ServiceOrderStatus::Completed;
        order.completed_at_ms = Some(now);
        order.updated_at_ms = now;
        let after = json_string(order);
        Ok(domain_action(
            "inspection.execute",
            "service_order.completed",
            "service_order",
            input.id,
            before,
            after,
            "Service order completed",
        ))
    }

    pub(crate) fn start_inspection(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "inspection.execute")?;
        let input: StartInspection = parse_data(payload)?;
        let (order_index, task_index) = self.find_task(&input.order_id, &input.task_id)?;
        if self.service_orders[order_index].status != ServiceOrderStatus::InProgress {
            return Err("service_order_not_in_progress".to_string());
        }
        let operator_id = input.operator_id.as_deref().unwrap_or(&payload.actor.id);
        self.ensure_actor_may_act_as(payload, operator_id)?;
        let operator = self.ensure_operator_level(
            operator_id,
            self.service_orders[order_index].tasks[task_index].complexity_snapshot,
        )?;
        let task = &self.service_orders[order_index].tasks[task_index];
        if task.status != ServiceOrderTaskStatus::Pending {
            return Err("service_order_task_already_started".to_string());
        }
        if task
            .assigned_operator_id
            .as_deref()
            .is_some_and(|assigned| assigned != operator_id)
        {
            return Err("service_order_task_assigned_to_other_operator".to_string());
        }
        let machine_item_id = task.machine_item_id.clone();
        let snapshot = task.item_snapshot.category.clone();
        let status_before = self
            .machine_items
            .iter()
            .find(|item| item.id == machine_item_id)
            .map(|item| item.status)
            .ok_or_else(|| "machine_item_not_found".to_string())?;
        let inspection_id = format!("inspection-{command_id}");
        let inspection = ItemInspection {
            id: inspection_id.clone(),
            service_order_task_id: Some(input.task_id.clone()),
            service_order_id: Some(input.order_id.clone()),
            machine_id: self.service_orders[order_index].machine_id.clone(),
            machine_item_id: machine_item_id.clone(),
            category_snapshot: snapshot,
            operator_id: operator.id.clone(),
            operator_name: operator.name,
            status: InspectionStatus::InProgress,
            started_at_ms: now,
            completed_at_ms: None,
            status_before,
            status_after: None,
            step_results: Vec::new(),
            findings: Vec::new(),
            photo_ids: Vec::new(),
            notes: None,
            maintenance_action: None,
        };
        let after = json_string(&inspection);
        self.inspections.push(inspection);
        let task = &mut self.service_orders[order_index].tasks[task_index];
        task.status = ServiceOrderTaskStatus::InProgress;
        task.assigned_operator_id = Some(operator.id);
        task.started_at_ms = Some(now);
        task.inspection_id = Some(inspection_id.clone());
        self.service_orders[order_index].updated_at_ms = now;
        if let Some(item) = self
            .machine_items
            .iter_mut()
            .find(|item| item.id == machine_item_id)
        {
            item.status = OperationalStatus::UnderMaintenance;
            item.updated_at_ms = now;
        }
        let machine_id = self.service_orders[order_index].machine_id.clone();
        self.refresh_machine_status(&machine_id, now);
        self.push_supplemental_audit(
            payload,
            now,
            "service_order_task.started",
            "service_order_task",
            &input.task_id,
            None,
            json_string(&self.service_orders[order_index].tasks[task_index]),
            "Service order task started",
        );
        Ok(domain_action(
            "inspection.execute",
            "inspection.started",
            "inspection",
            inspection_id,
            None,
            after,
            "Inspection started",
        ))
    }

    pub(crate) fn complete_inspection(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "inspection.execute")?;
        let input: CompleteInspection = parse_data(payload)?;
        let inspection_index = self
            .inspections
            .iter()
            .position(|inspection| inspection.id == input.id)
            .ok_or_else(|| "inspection_not_found".to_string())?;
        let inspection = &self.inspections[inspection_index];
        if inspection.status != InspectionStatus::InProgress {
            return Err("inspection_already_completed".to_string());
        }
        self.ensure_actor_may_act_as(payload, &inspection.operator_id)?;
        validate_step_results(
            &inspection.category_snapshot.maintenance_guide,
            &input.step_results,
            &self.photos,
            &input.id,
        )?;
        let before = json_string(inspection);
        let order_id = inspection
            .service_order_id
            .clone()
            .ok_or_else(|| "inspection_order_missing".to_string())?;
        let task_id = inspection
            .service_order_task_id
            .clone()
            .ok_or_else(|| "inspection_task_missing".to_string())?;
        let machine_id = inspection.machine_id.clone();
        let machine_item_id = inspection.machine_item_id.clone();
        let inspection = &mut self.inspections[inspection_index];
        inspection.status = InspectionStatus::Completed;
        inspection.completed_at_ms = Some(now);
        inspection.status_after = Some(input.status_after);
        inspection.step_results = input.step_results;
        inspection.findings = input.findings;
        inspection.photo_ids = input.photo_ids;
        inspection.notes = clean_optional(input.notes);
        inspection.maintenance_action = clean_optional(input.maintenance_action);
        let after = json_string(inspection);
        let (order_index, task_index) = self.find_task(&order_id, &task_id)?;
        let task = &mut self.service_orders[order_index].tasks[task_index];
        task.status = ServiceOrderTaskStatus::Completed;
        task.completed_at_ms = Some(now);
        self.service_orders[order_index].updated_at_ms = now;
        let previous_status = self
            .machine_items
            .iter()
            .find(|item| item.id == machine_item_id)
            .map(|item| item.status)
            .ok_or_else(|| "machine_item_not_found".to_string())?;
        if let Some(item) = self
            .machine_items
            .iter_mut()
            .find(|item| item.id == machine_item_id)
        {
            item.status = input.status_after;
            item.updated_at_ms = now;
        }
        self.refresh_machine_status(&machine_id, now);
        self.push_supplemental_audit(
            payload,
            now,
            "service_order_task.completed",
            "service_order_task",
            &task_id,
            None,
            json_string(&self.service_orders[order_index].tasks[task_index]),
            "Service order task completed",
        );
        self.push_supplemental_audit(
            payload,
            now,
            "item.status_changed",
            "machine_item",
            &machine_item_id,
            json_string(&previous_status),
            json_string(&input.status_after),
            "Machine item status changed after inspection",
        );
        Ok(domain_action(
            "inspection.execute",
            "inspection.completed",
            "inspection",
            input.id,
            before,
            after,
            "Inspection completed",
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

    fn ensure_operator_level(
        &self,
        operator_id: &str,
        required: ComplexityLevel,
    ) -> Result<UserAccount, String> {
        let operator = self
            .user_accounts
            .iter()
            .find(|user| user.id == operator_id && user.active)
            .ok_or_else(|| "operator_not_found_or_inactive".to_string())?;
        if !can_execute_complexity(operator.maximum_repair_level, required) {
            return Err("operator_repair_level_insufficient".to_string());
        }
        Ok(operator.clone())
    }

    fn ensure_actor_may_act_as(
        &self,
        payload: &CommandPayload,
        operator_id: &str,
    ) -> Result<(), String> {
        if payload.actor.id == operator_id
            || matches!(payload.actor.role, Role::Admin | Role::Chefe)
        {
            Ok(())
        } else {
            Err("operator_identity_mismatch".to_string())
        }
    }

    fn find_task(&self, order_id: &str, task_id: &str) -> Result<(usize, usize), String> {
        let order_index = self
            .service_orders
            .iter()
            .position(|order| order.id == order_id)
            .ok_or_else(|| "order_not_found".to_string())?;
        let task_index = self.service_orders[order_index]
            .tasks
            .iter()
            .position(|task| task.id == task_id)
            .ok_or_else(|| "service_order_task_not_found".to_string())?;
        Ok((order_index, task_index))
    }

    fn photo_permission(
        &self,
        owner_type: &PhotoOwnerType,
        owner_id: &str,
        payload: &CommandPayload,
    ) -> Result<&'static str, String> {
        let exists = match owner_type {
            PhotoOwnerType::Machine => self.machines.iter().any(|machine| machine.id == owner_id),
            PhotoOwnerType::MachineItem => {
                self.machine_items.iter().any(|item| item.id == owner_id)
            }
            PhotoOwnerType::GuideStep => self.item_categories.iter().any(|category| {
                category
                    .maintenance_guide
                    .steps
                    .iter()
                    .any(|step| step.id == owner_id)
            }),
            PhotoOwnerType::Inspection => self
                .inspections
                .iter()
                .any(|inspection| inspection.id == owner_id),
            PhotoOwnerType::Replacement => self
                .machine_item_replacements
                .iter()
                .any(|replacement| replacement.id == owner_id),
        };
        if !exists {
            return Err("photo_owner_not_found".to_string());
        }
        if *owner_type == PhotoOwnerType::Inspection {
            let inspection = self
                .inspections
                .iter()
                .find(|inspection| inspection.id == owner_id)
                .ok_or_else(|| "inspection_not_found".to_string())?;
            self.ensure_actor_may_act_as(payload, &inspection.operator_id)?;
            Ok("inspection.execute")
        } else {
            Ok("photo.manage_reference")
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn push_supplemental_audit(
        &mut self,
        payload: &CommandPayload,
        now: u64,
        operation: &str,
        aggregate: &str,
        aggregate_id: &str,
        before: Option<String>,
        after: Option<String>,
        description: &str,
    ) {
        self.audit_events.push(proexel_domain::AuditEvent {
            id: format!("audit-domain-{operation}-{aggregate_id}-{now}"),
            actor: payload.actor.name.clone(),
            role: role_name(payload.actor.role).to_string(),
            operation: operation.to_string(),
            aggregate: aggregate.to_string(),
            aggregate_id: aggregate_id.to_string(),
            description: Some(description.to_string()),
            trace_id: None,
            before_json: before,
            after_json: after,
            result: "success".to_string(),
            created_at_ms: now,
        });
    }
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Admin => "admin",
        Role::Chefe => "chefe",
        Role::Compras => "compras",
        Role::Tecnico => "tecnico",
    }
}

fn normalized_fields(mut fields: Vec<CustomFieldDefinition>) -> Vec<CustomFieldDefinition> {
    fields.sort_by_key(|field| field.order);
    fields
}

fn normalized_guide(mut guide: MaintenanceGuide) -> MaintenanceGuide {
    guide.steps.sort_by_key(|step| step.order);
    guide
}

fn validate_category_definition(
    fields: &[CustomFieldDefinition],
    guide: &MaintenanceGuide,
) -> Result<(), String> {
    let mut field_ids = BTreeSet::new();
    let mut field_keys = BTreeSet::new();
    for field in fields {
        require_text(&field.id, "custom_field_id_required")?;
        require_text(&field.key, "custom_field_key_required")?;
        require_text(&field.label, "custom_field_label_required")?;
        if !field_ids.insert(field.id.trim().to_string())
            || !field_keys.insert(field.key.trim().to_lowercase())
        {
            return Err("custom_field_duplicate".to_string());
        }
        if field.field_type == CustomFieldType::Choice && field.options.is_empty() {
            return Err("custom_field_choice_options_required".to_string());
        }
        if field
            .minimum
            .zip(field.maximum)
            .is_some_and(|(minimum, maximum)| minimum > maximum)
        {
            return Err("custom_field_range_invalid".to_string());
        }
    }
    let mut step_ids = BTreeSet::new();
    for step in &guide.steps {
        require_text(&step.id, "guide_step_id_required")?;
        require_text(&step.title, "guide_step_title_required")?;
        require_text(&step.instructions, "guide_step_instructions_required")?;
        if !step_ids.insert(step.id.trim().to_string()) {
            return Err("guide_step_duplicate".to_string());
        }
        if step.step_type == GuideStepType::Choice && step.options.is_empty() {
            return Err("guide_step_choice_options_required".to_string());
        }
        if matches!(
            step.step_type,
            GuideStepType::Numeric | GuideStepType::Measurement
        ) && step.expected_value.as_ref().is_some_and(|expected| {
            expected
                .minimum
                .zip(expected.maximum)
                .is_some_and(|(minimum, maximum)| minimum > maximum)
        }) {
            return Err("guide_step_expected_range_invalid".to_string());
        }
    }
    Ok(())
}

fn validate_custom_values(
    category: &ItemCategory,
    values: &BTreeMap<String, Value>,
) -> Result<(), String> {
    let definitions = category
        .custom_field_definitions
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<BTreeMap<_, _>>();
    if values
        .keys()
        .any(|key| !definitions.contains_key(key.as_str()))
    {
        return Err("custom_field_unknown".to_string());
    }
    for field in &category.custom_field_definitions {
        let value = values.get(&field.key);
        if field.required && value.is_none_or(is_empty_value) {
            return Err(format!("custom_field_required:{}", field.key));
        }
        let Some(value) = value.filter(|value| !is_empty_value(value)) else {
            continue;
        };
        let valid = match field.field_type {
            CustomFieldType::Text | CustomFieldType::Date => value.as_str().is_some(),
            CustomFieldType::Boolean => value.as_bool().is_some(),
            CustomFieldType::Choice => value
                .as_str()
                .is_some_and(|selected| field.options.iter().any(|option| option == selected)),
            CustomFieldType::Number => value.as_f64().is_some_and(|number| {
                field.minimum.is_none_or(|minimum| number >= minimum)
                    && field.maximum.is_none_or(|maximum| number <= maximum)
            }),
        };
        if !valid {
            return Err(format!("custom_field_value_invalid:{}", field.key));
        }
    }
    Ok(())
}

fn is_empty_value(value: &Value) -> bool {
    value.is_null() || value.as_str().is_some_and(|text| text.trim().is_empty())
}

fn item_snapshot(
    item: &MachineItem,
    category: &ItemCategory,
    photos: &[PhotoAsset],
) -> MachineItemSnapshot {
    let step_ids = category
        .maintenance_guide
        .steps
        .iter()
        .map(|step| step.id.as_str())
        .collect::<BTreeSet<_>>();
    MachineItemSnapshot {
        id: item.id.clone(),
        machine_id: item.machine_id.clone(),
        category: ItemCategorySnapshot {
            id: category.id.clone(),
            code: category.code.clone(),
            name: category.name.clone(),
            guide_version: category.maintenance_guide.version,
            maintenance_guide: category.maintenance_guide.clone(),
            guide_reference_photos: photos
                .iter()
                .filter(|photo| {
                    photo.owner_type == PhotoOwnerType::GuideStep
                        && step_ids.contains(photo.owner_id.as_str())
                        && category.maintenance_guide.steps.iter().any(|step| {
                            step.id == photo.owner_id
                                && step.reference_photo_ids.contains(&photo.id)
                        })
                })
                .cloned()
                .collect(),
        },
        name: item.name.clone(),
        code: item.code.clone(),
        complexity_level: item.complexity_level,
        location_description: item.location_description.clone(),
        installed_component: item.installed_component.clone(),
    }
}

fn validate_step_results(
    guide: &MaintenanceGuide,
    results: &[InspectionStepResult],
    photos: &[PhotoAsset],
    inspection_id: &str,
) -> Result<(), String> {
    let result_map = results
        .iter()
        .map(|result| (result.step_id.as_str(), result))
        .collect::<BTreeMap<_, _>>();
    if result_map.len() != results.len() {
        return Err("inspection_step_result_duplicate".to_string());
    }
    if results
        .iter()
        .any(|result| !guide.steps.iter().any(|step| step.id == result.step_id))
    {
        return Err("inspection_step_unknown".to_string());
    }
    for step in &guide.steps {
        let result = result_map.get(step.id.as_str()).copied();
        if step.required && result.is_none() {
            return Err(format!("inspection_step_required:{}", step.id));
        }
        let Some(result) = result else {
            continue;
        };
        let valid = match step.step_type {
            GuideStepType::Information | GuideStepType::Warning => true,
            GuideStepType::Confirmation | GuideStepType::Boolean => {
                result.value.as_bool().is_some()
            }
            GuideStepType::Choice => result
                .value
                .as_str()
                .is_some_and(|selected| step.options.iter().any(|option| option == selected)),
            GuideStepType::Numeric | GuideStepType::Measurement => result.value.as_f64().is_some(),
            GuideStepType::Text => result
                .value
                .as_str()
                .is_some_and(|text| !text.trim().is_empty()),
            GuideStepType::Photo => !result.photo_ids.is_empty(),
        };
        if !valid {
            return Err(format!("inspection_step_value_invalid:{}", step.id));
        }
        if step.step_type == GuideStepType::Measurement {
            let expected_unit = step
                .expected_value
                .as_ref()
                .and_then(|expected| expected.unit.as_deref());
            if expected_unit != result.unit.as_deref() {
                return Err(format!("inspection_measurement_unit_invalid:{}", step.id));
            }
        }
        for photo_id in &result.photo_ids {
            if !photos.iter().any(|photo| {
                photo.id == *photo_id
                    && photo.owner_type == PhotoOwnerType::Inspection
                    && photo.owner_id == inspection_id
            }) {
                return Err(format!("inspection_photo_invalid:{}", step.id));
            }
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct CategoryInput {
    code: String,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    default_complexity_level: ComplexityLevel,
    #[serde(default)]
    maintenance_guide: MaintenanceGuide,
    #[serde(default)]
    custom_field_definitions: Vec<CustomFieldDefinition>,
    #[serde(default)]
    recommended_parts: Vec<RecommendedPart>,
    #[serde(default = "default_true")]
    active: bool,
}

#[derive(Deserialize)]
struct UpdateCategory {
    id: String,
    #[serde(flatten)]
    category: CategoryInput,
}

#[derive(Deserialize)]
struct MachineInput {
    code: String,
    name: String,
    description: Option<String>,
    zone: String,
    location: Option<String>,
    manufacturer: Option<String>,
    model: Option<String>,
    serial_number: Option<String>,
    #[serde(default = "default_true")]
    active: bool,
}

#[derive(Deserialize)]
struct UpdateMachine {
    id: String,
    #[serde(flatten)]
    machine: MachineInput,
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

#[derive(Deserialize)]
struct AddPhoto {
    owner_type: PhotoOwnerType,
    owner_id: String,
    purpose: PhotoPurpose,
    blob_ref: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct DeletePhoto {
    id: String,
    blob_ref: String,
}

#[derive(Deserialize)]
struct CreateServiceOrder {
    machine_id: String,
    #[serde(default)]
    item_ids: Vec<String>,
    #[serde(default)]
    all_items: bool,
    description: String,
    priority: ServiceOrderPriority,
    scheduled_for: Option<String>,
    assigned_operator_id: Option<String>,
}

#[derive(Deserialize)]
struct StartServiceOrder {
    id: String,
    operator_id: Option<String>,
}

#[derive(Deserialize)]
struct AssignOrderTask {
    order_id: String,
    task_id: String,
    operator_id: String,
}

#[derive(Deserialize)]
struct CompleteServiceOrder {
    id: String,
    operator_id: Option<String>,
}

#[derive(Deserialize)]
struct StartInspection {
    order_id: String,
    task_id: String,
    operator_id: Option<String>,
}

#[derive(Deserialize)]
struct CompleteInspection {
    id: String,
    status_after: OperationalStatus,
    #[serde(default)]
    step_results: Vec<InspectionStepResult>,
    #[serde(default)]
    findings: Vec<InspectionFinding>,
    #[serde(default)]
    photo_ids: Vec<String>,
    notes: Option<String>,
    maintenance_action: Option<String>,
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands;

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
            maximum_repair_level: ComplexityLevel::new(5).unwrap(),
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
}
