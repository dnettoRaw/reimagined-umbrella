use proexel_domain::{
    InspectionFinding, InspectionStatus, InspectionStepResult, ItemInspection, OperationalStatus,
    ServiceOrderStatus, ServiceOrderTaskStatus,
};
use serde::Deserialize;

use crate::{
    state::{
        clean_optional, domain_action, json_string, parse_data, require_permission, Action,
        CommandPayload,
    },
    ApplicationState,
};

use crate::asset_audit::SupplementalAudit;
use crate::asset_validation::validate_step_results;

impl ApplicationState {
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
            SupplementalAudit {
                operation: "service_order_task.started",
                aggregate: "service_order_task",
                aggregate_id: &input.task_id,
                before: None,
                after: json_string(&self.service_orders[order_index].tasks[task_index]),
                description: "Service order task started",
            },
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
            SupplementalAudit {
                operation: "service_order_task.completed",
                aggregate: "service_order_task",
                aggregate_id: &task_id,
                before: None,
                after: json_string(&self.service_orders[order_index].tasks[task_index]),
                description: "Service order task completed",
            },
        );
        self.push_supplemental_audit(
            payload,
            now,
            SupplementalAudit {
                operation: "item.status_changed",
                aggregate: "machine_item",
                aggregate_id: &machine_item_id,
                before: json_string(&previous_status),
                after: json_string(&input.status_after),
                description: "Machine item status changed after inspection",
            },
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
