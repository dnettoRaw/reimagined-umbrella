use proexel_domain::{
    adjust_stock, RestockRequest, RestockStatus, StockItem, StockMovement, StockMovementKind,
};
use serde::Deserialize;

use crate::{
    state::{
        action, clean_optional, json_string, parse_data, require_permission, require_text, Action,
        CommandPayload,
    },
    ApplicationState,
};

impl ApplicationState {
    pub(crate) fn create_restock_request(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "restock.create_suggestion")?;
        let input: CreateRestockRequest = parse_data(payload)?;
        let reference = proexel_domain::normalize_reference(&input.reference);
        require_text(&reference, "reference_required")?;
        require_text(&input.reason, "reason_required")?;
        let id = format!("restock-{command_id}");
        let request = RestockRequest {
            id: id.clone(),
            reference,
            reason: input.reason.trim().to_string(),
            requested_by: payload.actor.name.clone(),
            status: RestockStatus::Pending,
            reviewed_by: None,
            reviewed_at_ms: None,
            created_at_ms: now,
        };
        let after = json_string(&request);
        self.restock_requests.push(request);
        Ok(action(
            "restock.create_suggestion",
            "restock_request",
            id,
            None,
            after,
            "Restock requested",
        ))
    }

    pub(crate) fn review_restock_request(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "restock.approve_reject")?;
        let input: ReviewRestockRequest = parse_data(payload)?;
        if input.status == RestockStatus::Pending {
            return Err("review_status_must_be_final".to_string());
        }
        let request = self
            .restock_requests
            .iter_mut()
            .find(|request| request.id == input.id)
            .ok_or_else(|| "restock_request_not_found".to_string())?;
        if request.status != RestockStatus::Pending {
            return Err("restock_request_already_reviewed".to_string());
        }
        let before = json_string(request);
        request.status = input.status;
        request.reviewed_by = Some(payload.actor.name.clone());
        request.reviewed_at_ms = Some(now);
        let after = json_string(request);
        Ok(action(
            "restock.approve_reject",
            "restock_request",
            input.id,
            before,
            after,
            "Restock request reviewed",
        ))
    }

    pub(crate) fn delete_restock_request(
        &mut self,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "restock.delete")?;
        let input: DeleteById = parse_data(payload)?;
        let index = self
            .restock_requests
            .iter()
            .position(|request| request.id == input.id)
            .ok_or_else(|| "restock_request_not_found".to_string())?;
        if self.restock_requests[index].status == RestockStatus::Approved {
            return Err("approved_restock_cannot_be_deleted".to_string());
        }
        let request = self.restock_requests.remove(index);
        Ok(action(
            "restock.delete",
            "restock_request",
            request.id.clone(),
            json_string(&request),
            None,
            "Restock request deleted",
        ))
    }

    pub(crate) fn adjust_stock_item(
        &mut self,
        command_id: &str,
        idempotency_key: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "stock.adjust_quantity")?;
        let input: AdjustStock = parse_data(payload)?;
        require_text(&input.reason, "adjustment_reason_required")?;
        if input.delta == 0 {
            return Err("adjustment_delta_cannot_be_zero".to_string());
        }
        let item = self
            .stock_items
            .iter_mut()
            .find(|item| item.id == input.id)
            .ok_or_else(|| "stock_item_not_found".to_string())?;
        let before = json_string(item);
        item.quantity = adjust_stock(item.quantity, input.delta).map_err(str::to_string)?;
        item.updated_at_ms = now;
        let after = json_string(item);
        self.stock_movements.push(StockMovement {
            id: format!("movement-{command_id}"),
            stock_item_id: item.id.clone(),
            kind: StockMovementKind::Correction,
            delta: input.delta,
            balance_after: item.quantity,
            reason: input.reason,
            actor: payload.actor.name.clone(),
            idempotency_key: idempotency_key.to_string(),
            created_at_ms: now,
        });
        Ok(action(
            "stock.adjust_quantity",
            "stock_item",
            input.id,
            before,
            after,
            "Stock adjusted",
        ))
    }

    pub(crate) fn upsert_stock_item(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "stock.add_or_increment")?;
        let input: UpsertStock = parse_data(payload)?;
        let reference = proexel_domain::normalize_reference(&input.reference);
        require_text(&reference, "reference_required")?;
        if let Some(item) = self
            .stock_items
            .iter_mut()
            .find(|item| item.reference_normalized == reference)
        {
            let before = json_string(item);
            item.minimum_quantity = input.minimum_quantity;
            item.manufacturer = clean_optional(input.manufacturer);
            item.location = clean_optional(input.location);
            item.updated_at_ms = now;
            let after = json_string(item);
            return Ok(action(
                "stock.add_or_increment",
                "stock_item",
                item.id.clone(),
                before,
                after,
                "Stock item updated",
            ));
        }
        let id = format!("stock-{command_id}");
        let item = StockItem {
            id: id.clone(),
            reference: reference.clone(),
            reference_normalized: reference,
            quantity: 0,
            minimum_quantity: input.minimum_quantity,
            manufacturer: clean_optional(input.manufacturer),
            location: clean_optional(input.location),
            created_at_ms: now,
            updated_at_ms: now,
        };
        let after = json_string(&item);
        self.stock_items.push(item);
        Ok(action(
            "stock.add_or_increment",
            "stock_item",
            id,
            None,
            after,
            "Stock item created",
        ))
    }

    pub(crate) fn delete_stock_item(&mut self, payload: &CommandPayload) -> Result<Action, String> {
        require_permission(payload.actor.role, "stock.delete")?;
        let input: DeleteById = parse_data(payload)?;
        let index = self
            .stock_items
            .iter()
            .position(|item| item.id == input.id)
            .ok_or_else(|| "stock_item_not_found".to_string())?;
        if self.stock_items[index].quantity > 0 {
            return Err("stock_item_not_empty".to_string());
        }
        let item = self.stock_items.remove(index);
        Ok(action(
            "stock.delete",
            "stock_item",
            item.id.clone(),
            json_string(&item),
            None,
            "Stock item deleted",
        ))
    }
}

#[derive(Deserialize)]
struct DeleteById {
    id: String,
}

#[derive(Deserialize)]
struct CreateRestockRequest {
    reference: String,
    reason: String,
}

#[derive(Deserialize)]
struct ReviewRestockRequest {
    id: String,
    status: RestockStatus,
}

#[derive(Deserialize)]
struct AdjustStock {
    id: String,
    delta: i32,
    reason: String,
}

#[derive(Deserialize)]
struct UpsertStock {
    reference: String,
    #[serde(default)]
    minimum_quantity: u32,
    manufacturer: Option<String>,
    location: Option<String>,
}
