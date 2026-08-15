use proexel_domain::{can_execute_complexity, ComplexityLevel, UserAccount};

use crate::{state::CommandPayload, ApplicationState, Role};

impl ApplicationState {
    pub(crate) fn ensure_operator_level(
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

    pub(crate) fn ensure_actor_may_act_as(
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

    pub(crate) fn find_task(
        &self,
        order_id: &str,
        task_id: &str,
    ) -> Result<(usize, usize), String> {
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
}
