use std::collections::BTreeSet;

use proexel_domain::{
    MachineSnapshot, ServiceOrder, ServiceOrderPriority, ServiceOrderStatus, ServiceOrderTask,
    ServiceOrderTaskStatus,
};
use serde::Deserialize;

use crate::{
    state::{
        clean_optional, domain_action, json_string, parse_data, require_permission, require_text,
        Action, CommandPayload,
    },
    ApplicationState,
};

use crate::asset_snapshot::item_snapshot;

impl ApplicationState {
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
                location: machine.location,
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
        let input: DeleteServiceOrder = parse_data(payload)?;
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
struct DeleteServiceOrder {
    id: String,
}
