use proexel_application::ApplicationState;
use proexel_domain::{
    normalize_tag, InspectionStatus, ItemInspection, OperationalStatus, ServiceOrder,
    ServiceOrderStatus, ServiceOrderTask, ServiceOrderTaskStatus,
};

use crate::{
    migration_support::{
        category_snapshot, clean, date_to_ms, find_item_by_tag, hash, item_snapshot,
        machine_for_zone, machine_snapshot, order_priority, order_status,
    },
    report::inc,
    LegacyBundle, MigrationReport,
};

pub(crate) fn import_inspections(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for (index, old) in bundle.maintenance_records.iter().enumerate() {
        let tag = normalize_tag(&old.tag);
        let Some(item) = find_item_by_tag(state, &tag).cloned() else {
            report
                .warnings
                .push(format!("maintenance {index} skipped: item {tag} not found"));
            continue;
        };
        let id = format!(
            "legacy-inspection-{}",
            hash(&format!(
                "{tag}|{}|{}|{}",
                old.performed_at, old.technician, old.service
            ))
        );
        if state
            .inspections
            .iter()
            .any(|inspection| inspection.id == id)
        {
            continue;
        }
        let Some(category) = state
            .item_categories
            .iter()
            .find(|category| category.id == item.category_id)
        else {
            report
                .warnings
                .push(format!("maintenance {index} skipped: category not found"));
            continue;
        };
        let notes = [
            clean(&old.notes),
            Some(format!("Legacy type: {}", old.maintenance_type.trim())),
            old.kit_changed
                .then(|| "Legacy kit replacement: yes".to_string()),
            clean(&old.signature_ref).map(|value| format!("Legacy signature: {value}")),
            Some(format!("Legacy performed at: {}", old.performed_at.trim())),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n");
        let performed_at_ms = date_to_ms(&old.performed_at).unwrap_or(now);
        state.inspections.push(ItemInspection {
            id,
            service_order_task_id: None,
            service_order_id: None,
            machine_id: item.machine_id.clone(),
            machine_item_id: item.id,
            category_snapshot: category_snapshot(category),
            operator_id: format!("legacy-operator-{}", hash(&old.technician.to_lowercase())),
            operator_name: old.technician.trim().to_string(),
            status: InspectionStatus::Completed,
            started_at_ms: performed_at_ms,
            completed_at_ms: Some(performed_at_ms),
            status_before: OperationalStatus::Unknown,
            status_after: Some(OperationalStatus::Unknown),
            step_results: Vec::new(),
            findings: Vec::new(),
            photo_ids: Vec::new(),
            notes: (!notes.is_empty()).then_some(notes),
            maintenance_action: clean(&Some(old.service.clone())),
        });
        inc(report, "inspections");
    }
}

pub(crate) fn import_orders(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.orders {
        let Some(machine) = machine_for_zone(state, &old.zone).cloned() else {
            report.warnings.push(format!(
                "order skipped: machine for zone {} not found",
                old.zone
            ));
            continue;
        };
        let id = format!(
            "legacy-order-{}",
            hash(&format!(
                "{}|{}|{}",
                old.zone,
                old.description,
                old.scheduled_for.as_deref().unwrap_or("")
            ))
        );
        if state.service_orders.iter().any(|order| order.id == id) {
            continue;
        }
        let selected_items = if let Some(tag) = old.valve_tag.as_deref() {
            let normalized = normalize_tag(tag);
            state
                .machine_items
                .iter()
                .filter(|item| {
                    item.machine_id == machine.id
                        && item.code_normalized == normalized
                        && item.active
                })
                .collect::<Vec<_>>()
        } else {
            state
                .machine_items
                .iter()
                .filter(|item| item.machine_id == machine.id && item.active)
                .collect::<Vec<_>>()
        };
        if selected_items.is_empty() {
            report
                .warnings
                .push(format!("order {id} skipped: no target items"));
            continue;
        }
        let status = order_status(&old.status);
        let task_status = match status {
            ServiceOrderStatus::Completed => ServiceOrderTaskStatus::Completed,
            ServiceOrderStatus::InProgress => ServiceOrderTaskStatus::InProgress,
            ServiceOrderStatus::Pending | ServiceOrderStatus::Cancelled => {
                ServiceOrderTaskStatus::Pending
            }
        };
        let tasks = selected_items
            .into_iter()
            .filter_map(|item| {
                let category = state
                    .item_categories
                    .iter()
                    .find(|category| category.id == item.category_id)?;
                Some(ServiceOrderTask {
                    id: format!("{id}-task-{}", hash(&item.id)),
                    machine_item_id: item.id.clone(),
                    item_snapshot: item_snapshot(item, category),
                    complexity_snapshot: item.complexity_level,
                    assigned_operator_id: None,
                    status: task_status,
                    started_at_ms: (task_status != ServiceOrderTaskStatus::Pending).then_some(now),
                    completed_at_ms: (task_status == ServiceOrderTaskStatus::Completed)
                        .then_some(now),
                    inspection_id: None,
                })
            })
            .collect::<Vec<_>>();
        let description = match clean(&old.technician) {
            Some(technician) => format!(
                "{}\nLegacy technician: {technician}",
                old.description.trim()
            ),
            None => old.description.trim().to_string(),
        };
        state.service_orders.push(ServiceOrder {
            id,
            machine_id: machine.id.clone(),
            machine_snapshot: machine_snapshot(&machine),
            description,
            priority: order_priority(&old.priority),
            status,
            created_by: old
                .created_by
                .clone()
                .unwrap_or_else(|| "legacy".to_string()),
            scheduled_for: clean(&old.scheduled_for),
            tasks,
            created_at_ms: now,
            started_at_ms: (!matches!(status, ServiceOrderStatus::Pending)).then_some(now),
            completed_at_ms: (status == ServiceOrderStatus::Completed).then_some(now),
            updated_at_ms: now,
        });
        inc(report, "service_orders");
    }
}
