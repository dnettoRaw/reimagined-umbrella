use proexel_domain::PhotoOwnerType;

use crate::{state::CommandPayload, ApplicationState, Role};

pub(crate) struct SupplementalAudit<'a> {
    pub operation: &'a str,
    pub aggregate: &'a str,
    pub aggregate_id: &'a str,
    pub before: Option<String>,
    pub after: Option<String>,
    pub description: &'a str,
}

impl ApplicationState {
    pub(crate) fn photo_permission(
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

    pub(crate) fn push_supplemental_audit(
        &mut self,
        payload: &CommandPayload,
        now: u64,
        audit: SupplementalAudit<'_>,
    ) {
        self.audit_events.push(proexel_domain::AuditEvent {
            id: format!(
                "audit-domain-{}-{}-{now}",
                audit.operation, audit.aggregate_id
            ),
            actor: payload.actor.name.clone(),
            role: role_name(payload.actor.role).to_string(),
            operation: audit.operation.to_string(),
            aggregate: audit.aggregate.to_string(),
            aggregate_id: audit.aggregate_id.to_string(),
            description: Some(audit.description.to_string()),
            trace_id: None,
            before_json: audit.before,
            after_json: audit.after,
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
