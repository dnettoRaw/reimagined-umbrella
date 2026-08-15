use std::collections::BTreeMap;

use proexel_domain::{
    adjust_stock, can_transition_order, normalize_reference, normalize_tag, AuditEvent,
    MaintenanceRecord, MaintenanceType, RestockRequest, RestockStatus, ServiceOrder,
    ServiceOrderPriority, ServiceOrderStatus, StockItem, StockMovement, StockMovementKind,
    Supplier, Valve, ValvePhoto,
};
use serde::{Deserialize, Serialize};

use crate::{commands, permissions::can, Role};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub id: String,
    pub name: String,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPayload {
    pub actor: Actor,
    #[serde(default)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub aggregate: String,
    pub aggregate_id: String,
    pub event_name: String,
    pub replayed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationState {
    pub schema_version: u32,
    #[serde(default)]
    pub valves: Vec<Valve>,
    #[serde(default)]
    pub maintenance_records: Vec<MaintenanceRecord>,
    #[serde(default)]
    pub service_orders: Vec<ServiceOrder>,
    #[serde(default)]
    pub restock_requests: Vec<RestockRequest>,
    #[serde(default)]
    pub stock_items: Vec<StockItem>,
    #[serde(default)]
    pub stock_movements: Vec<StockMovement>,
    #[serde(default)]
    pub suppliers: Vec<Supplier>,
    #[serde(default)]
    pub valve_photos: Vec<ValvePhoto>,
    #[serde(default)]
    pub audit_events: Vec<AuditEvent>,
    #[serde(default)]
    pub processed_commands: BTreeMap<String, ExecutionReceipt>,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            valves: Vec::new(),
            maintenance_records: Vec::new(),
            service_orders: Vec::new(),
            restock_requests: Vec::new(),
            stock_items: Vec::new(),
            stock_movements: Vec::new(),
            suppliers: Vec::new(),
            valve_photos: Vec::new(),
            audit_events: Vec::new(),
            processed_commands: BTreeMap::new(),
        }
    }
}

impl ApplicationState {
    pub fn execute(
        &mut self,
        command_name: &str,
        command_id: &str,
        idempotency_key: &str,
        issued_at_ms: u64,
        raw_payload: &[u8],
    ) -> Result<ExecutionReceipt, String> {
        if let Some(receipt) = self.processed_commands.get(idempotency_key) {
            let mut replayed = receipt.clone();
            replayed.replayed = true;
            return Ok(replayed);
        }

        let payload: CommandPayload =
            serde_json::from_slice(raw_payload).map_err(|_| "invalid_json_payload".to_string())?;
        validate_actor(&payload.actor)?;

        let (permission, receipt, before, after, description) = match command_name {
            commands::CREATE_VALVE => self.create_valve(command_id, issued_at_ms, &payload)?,
            commands::UPDATE_VALVE => self.update_valve(issued_at_ms, &payload)?,
            commands::ADD_VALVE_PHOTO => self.add_valve_photo(command_id, &payload)?,
            commands::DELETE_VALVE_PHOTO => self.delete_valve_photo(&payload)?,
            commands::REGISTER_MAINTENANCE => {
                self.register_maintenance(command_id, idempotency_key, issued_at_ms, &payload)?
            }
            commands::CREATE_SERVICE_ORDER => {
                self.create_service_order(command_id, issued_at_ms, &payload)?
            }
            commands::CHANGE_SERVICE_ORDER_STATUS => {
                self.change_service_order_status(issued_at_ms, &payload)?
            }
            commands::DELETE_SERVICE_ORDER => self.delete_service_order(&payload)?,
            commands::CREATE_RESTOCK_REQUEST => {
                self.create_restock_request(command_id, issued_at_ms, &payload)?
            }
            commands::REVIEW_RESTOCK_REQUEST => {
                self.review_restock_request(issued_at_ms, &payload)?
            }
            commands::DELETE_RESTOCK_REQUEST => self.delete_restock_request(&payload)?,
            commands::ADJUST_STOCK => {
                self.adjust_stock_item(command_id, idempotency_key, issued_at_ms, &payload)?
            }
            commands::UPSERT_STOCK_ITEM => {
                self.upsert_stock_item(command_id, issued_at_ms, &payload)?
            }
            commands::DELETE_STOCK_ITEM => self.delete_stock_item(&payload)?,
            commands::CREATE_SUPPLIER => {
                self.create_supplier(command_id, issued_at_ms, &payload)?
            }
            commands::UPDATE_SUPPLIER => self.update_supplier(issued_at_ms, &payload)?,
            commands::DELETE_SUPPLIER => self.delete_supplier(&payload)?,
            _ => return Err("unknown_command".to_string()),
        };

        if !can(payload.actor.role, permission) {
            return Err("forbidden".to_string());
        }

        self.audit_events.push(AuditEvent {
            id: format!("audit-{command_id}"),
            actor: payload.actor.name,
            role: role_name(payload.actor.role).to_string(),
            operation: command_name.to_string(),
            aggregate: receipt.aggregate.clone(),
            aggregate_id: receipt.aggregate_id.clone(),
            description: Some(description),
            trace_id: Some(command_id.to_string()),
            before_json: before,
            after_json: after,
            result: "success".to_string(),
            created_at_ms: issued_at_ms,
        });
        self.processed_commands
            .insert(idempotency_key.to_string(), receipt.clone());
        Ok(receipt)
    }

    fn create_valve(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "valve.create")?;
        let input: CreateValve = parse_data(payload)?;
        let tag = normalize_tag(&input.tag);
        require_text(&tag, "tag_required")?;
        require_text(&input.zone, "zone_required")?;
        if self.valves.iter().any(|valve| valve.tag_normalized == tag) {
            return Err("tag_already_exists".to_string());
        }
        let id = format!("valve-{command_id}");
        let kit_reference = normalized_optional(input.kit_reference);
        let valve = Valve {
            id: id.clone(),
            tag: tag.clone(),
            tag_normalized: tag,
            zone: input.zone.trim().to_string(),
            manufacturer: clean_optional(input.manufacturer),
            serial: clean_optional(input.serial),
            kit_reference: kit_reference.clone(),
            seat: clean_optional(input.seat),
            dn: clean_optional(input.dn),
            valve_type: clean_optional(input.valve_type),
            actuator: clean_optional(input.actuator),
            manufactured_at: clean_optional(input.manufactured_at),
            last_kit_changed_at: clean_optional(input.last_kit_changed_at),
            last_maintenance_at: clean_optional(input.last_maintenance_at),
            created_at_ms: now,
            updated_at_ms: now,
        };
        if let Some(reference) = kit_reference {
            self.ensure_stock_item(&reference, command_id, now);
        }
        let after = json_string(&valve);
        self.valves.push(valve);
        Ok(action(
            "valve.create",
            "valve",
            id,
            None,
            after,
            "Valve created",
        ))
    }

    fn update_valve(&mut self, now: u64, payload: &CommandPayload) -> Result<Action, String> {
        require_permission(payload.actor.role, "valve.update_technical_fields")?;
        let input: UpdateValve = parse_data(payload)?;
        let existing_index = self
            .valves
            .iter()
            .position(|valve| valve.id == input.id)
            .ok_or_else(|| "valve_not_found".to_string())?;
        let before = json_string(&self.valves[existing_index]);
        let new_tag = input.tag.as_deref().map(normalize_tag);
        if let Some(tag) = new_tag.as_ref() {
            require_text(tag, "tag_required")?;
            if self
                .valves
                .iter()
                .any(|valve| valve.id != input.id && valve.tag_normalized == *tag)
            {
                return Err("tag_already_exists".to_string());
            }
        }
        let new_kit = input.kit_reference.map(|value| normalize_reference(&value));
        if let Some(reference) = new_kit.as_ref().filter(|value| !value.is_empty()) {
            self.ensure_stock_item(reference, &input.id, now);
        }
        let valve = &mut self.valves[existing_index];
        if let Some(tag) = new_tag {
            valve.tag = tag.clone();
            valve.tag_normalized = tag;
        }
        set_if_some(
            &mut valve.zone,
            input.zone.map(|value| value.trim().to_string()),
        );
        set_optional(&mut valve.manufacturer, input.manufacturer);
        set_optional(&mut valve.serial, input.serial);
        if let Some(reference) = new_kit {
            valve.kit_reference = (!reference.is_empty()).then_some(reference);
        }
        set_optional(&mut valve.seat, input.seat);
        set_optional(&mut valve.dn, input.dn);
        set_optional(&mut valve.valve_type, input.valve_type);
        set_optional(&mut valve.actuator, input.actuator);
        set_optional(&mut valve.manufactured_at, input.manufactured_at);
        valve.updated_at_ms = now;
        let after = json_string(valve);
        Ok(action(
            "valve.update_technical_fields",
            "valve",
            input.id,
            before,
            after,
            "Valve updated",
        ))
    }

    fn add_valve_photo(
        &mut self,
        command_id: &str,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "valve.update_photo")?;
        let input: AddValvePhoto = parse_data(payload)?;
        require_text(&input.blob_ref, "photo_ref_required")?;
        if !self.valves.iter().any(|valve| valve.id == input.valve_id) {
            return Err("valve_not_found".to_string());
        }
        if self
            .valve_photos
            .iter()
            .any(|photo| photo.blob_ref == input.blob_ref)
        {
            return Err("photo_already_exists".to_string());
        }
        let id = format!("photo-{command_id}");
        let photo = ValvePhoto {
            id: id.clone(),
            valve_id: input.valve_id,
            legacy_tag: None,
            blob_ref: input.blob_ref.trim().to_string(),
        };
        let after = json_string(&photo);
        self.valve_photos.push(photo);
        Ok(action(
            "valve.update_photo",
            "valve_photo",
            id,
            None,
            after,
            "Valve photo added",
        ))
    }

    fn delete_valve_photo(&mut self, payload: &CommandPayload) -> Result<Action, String> {
        require_permission(payload.actor.role, "valve.update_photo")?;
        let input: DeleteById = parse_data(payload)?;
        let index = self
            .valve_photos
            .iter()
            .position(|photo| photo.id == input.id)
            .ok_or_else(|| "photo_not_found".to_string())?;
        let photo = self.valve_photos.remove(index);
        Ok(action(
            "valve.update_photo",
            "valve_photo",
            photo.id.clone(),
            json_string(&photo),
            None,
            "Valve photo removed",
        ))
    }

    fn register_maintenance(
        &mut self,
        command_id: &str,
        idempotency_key: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "maintenance.register")?;
        let input: RegisterMaintenance = parse_data(payload)?;
        require_text(&input.performed_at, "performed_at_required")?;
        require_text(&input.technician, "technician_required")?;
        require_text(&input.service, "service_required")?;
        let signature_ref = clean_optional(input.signature_ref)
            .filter(|value| value.starts_with("signatures/"))
            .ok_or_else(|| "signature_required".to_string())?;
        let valve_index = self
            .valves
            .iter()
            .position(|valve| valve.id == input.valve_id)
            .ok_or_else(|| "valve_not_found".to_string())?;
        let before = json_string(&self.valves[valve_index]);
        let kit_reference = self.valves[valve_index].kit_reference.clone();
        let mut stock_consumed = false;
        let mut stock_consumption_pending = false;
        if input.kit_changed {
            if let Some(reference) = kit_reference.as_ref() {
                if let Some(item) = self
                    .stock_items
                    .iter_mut()
                    .find(|item| item.reference_normalized == *reference)
                {
                    if item.quantity > 0 {
                        item.quantity -= 1;
                        item.updated_at_ms = now;
                        stock_consumed = true;
                        self.stock_movements.push(StockMovement {
                            id: format!("movement-{command_id}"),
                            stock_item_id: item.id.clone(),
                            kind: StockMovementKind::Consumption,
                            delta: -1,
                            balance_after: item.quantity,
                            reason: format!("Maintenance on {}", self.valves[valve_index].tag),
                            actor: payload.actor.name.clone(),
                            idempotency_key: idempotency_key.to_string(),
                            created_at_ms: now,
                        });
                    } else {
                        stock_consumption_pending = true;
                    }
                } else {
                    stock_consumption_pending = true;
                }
            } else {
                stock_consumption_pending = true;
            }
        }
        let valve = &mut self.valves[valve_index];
        valve.last_maintenance_at = Some(input.performed_at.clone());
        if input.kit_changed {
            valve.last_kit_changed_at = Some(input.performed_at.clone());
        }
        valve.updated_at_ms = now;
        let id = format!("maintenance-{command_id}");
        self.maintenance_records.push(MaintenanceRecord {
            id: id.clone(),
            valve_id: input.valve_id,
            valve_tag_snapshot: valve.tag.clone(),
            performed_at: input.performed_at,
            technician: input.technician,
            maintenance_type: input.maintenance_type,
            service: input.service,
            notes: clean_optional(input.notes),
            signature_ref: Some(signature_ref),
            kit_changed: input.kit_changed,
            kit_reference_snapshot: kit_reference,
            stock_consumed,
            stock_consumption_pending,
            idempotency_key: idempotency_key.to_string(),
            created_at_ms: now,
        });
        let after = json_string(valve);
        Ok(action(
            "maintenance.register",
            "maintenance",
            id,
            before,
            after,
            "Maintenance registered",
        ))
    }

    fn create_service_order(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "order.create")?;
        let input: CreateServiceOrder = parse_data(payload)?;
        require_text(&input.zone, "zone_required")?;
        require_text(&input.description, "description_required")?;
        if let Some(valve_id) = input.valve_id.as_ref() {
            if !self.valves.iter().any(|valve| valve.id == *valve_id) {
                return Err("valve_not_found".to_string());
            }
        }
        let valve_tag_snapshot = input
            .valve_id
            .as_ref()
            .and_then(|id| self.valves.iter().find(|v| v.id == *id))
            .map(|v| v.tag.clone());
        let id = format!("order-{command_id}");
        let order = ServiceOrder {
            id: id.clone(),
            zone: input.zone.trim().to_string(),
            valve_id: input.valve_id,
            valve_tag_snapshot,
            description: input.description.trim().to_string(),
            priority: input.priority,
            status: ServiceOrderStatus::Pending,
            created_by: payload.actor.name.clone(),
            technician: clean_optional(input.technician),
            scheduled_for: clean_optional(input.scheduled_for),
            created_at_ms: now,
            updated_at_ms: now,
        };
        let after = json_string(&order);
        self.service_orders.push(order);
        Ok(action(
            "order.create",
            "service_order",
            id,
            None,
            after,
            "Service order created",
        ))
    }

    fn change_service_order_status(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "order.change_status")?;
        let input: ChangeServiceOrderStatus = parse_data(payload)?;
        let order = self
            .service_orders
            .iter_mut()
            .find(|order| order.id == input.id)
            .ok_or_else(|| "order_not_found".to_string())?;
        if !can_transition_order(order.status, input.status) {
            return Err("invalid_order_status_transition".to_string());
        }
        let before = json_string(order);
        order.status = input.status;
        order.updated_at_ms = now;
        let after = json_string(order);
        Ok(action(
            "order.change_status",
            "service_order",
            input.id,
            before,
            after,
            "Service order status changed",
        ))
    }

    fn delete_service_order(&mut self, payload: &CommandPayload) -> Result<Action, String> {
        require_permission(payload.actor.role, "order.delete")?;
        let input: DeleteById = parse_data(payload)?;
        let index = self
            .service_orders
            .iter()
            .position(|order| order.id == input.id)
            .ok_or_else(|| "order_not_found".to_string())?;
        if self.service_orders[index].status == ServiceOrderStatus::Completed {
            return Err("completed_order_cannot_be_deleted".to_string());
        }
        let order = self.service_orders.remove(index);
        Ok(action(
            "order.delete",
            "service_order",
            order.id.clone(),
            json_string(&order),
            None,
            "Service order deleted",
        ))
    }

    fn create_restock_request(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "restock.create_suggestion")?;
        let input: CreateRestockRequest = parse_data(payload)?;
        let reference = normalize_reference(&input.reference);
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

    fn review_restock_request(
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

    fn delete_restock_request(&mut self, payload: &CommandPayload) -> Result<Action, String> {
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

    fn adjust_stock_item(
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

    fn upsert_stock_item(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "stock.add_or_increment")?;
        let input: UpsertStock = parse_data(payload)?;
        let reference = normalize_reference(&input.reference);
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

    fn delete_stock_item(&mut self, payload: &CommandPayload) -> Result<Action, String> {
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

    fn create_supplier(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "supplier.create_update_delete")?;
        let input: SupplierInput = parse_data(payload)?;
        require_text(&input.name, "supplier_name_required")?;
        require_text(&input.contact, "supplier_contact_required")?;
        validate_supplier_links(input.email.as_deref(), input.website.as_deref())?;
        let id = format!("supplier-{command_id}");
        let supplier = Supplier {
            id: id.clone(),
            name: input.name.trim().to_string(),
            contact: input.contact.trim().to_string(),
            email: clean_optional(input.email),
            website: clean_optional(input.website),
            notes: clean_optional(input.notes),
            created_by: payload.actor.name.clone(),
            created_at_ms: now,
            updated_at_ms: now,
        };
        let after = json_string(&supplier);
        self.suppliers.push(supplier);
        Ok(action(
            "supplier.create_update_delete",
            "supplier",
            id,
            None,
            after,
            "Supplier created",
        ))
    }

    fn update_supplier(&mut self, now: u64, payload: &CommandPayload) -> Result<Action, String> {
        require_permission(payload.actor.role, "supplier.create_update_delete")?;
        let input: UpdateSupplier = parse_data(payload)?;
        require_text(&input.name, "supplier_name_required")?;
        require_text(&input.contact, "supplier_contact_required")?;
        validate_supplier_links(input.email.as_deref(), input.website.as_deref())?;
        let supplier = self
            .suppliers
            .iter_mut()
            .find(|supplier| supplier.id == input.id)
            .ok_or_else(|| "supplier_not_found".to_string())?;
        let before = json_string(supplier);
        supplier.name = input.name.trim().to_string();
        supplier.contact = input.contact.trim().to_string();
        supplier.email = clean_optional(input.email);
        supplier.website = clean_optional(input.website);
        supplier.notes = clean_optional(input.notes);
        supplier.updated_at_ms = now;
        let after = json_string(supplier);
        Ok(action(
            "supplier.create_update_delete",
            "supplier",
            input.id,
            before,
            after,
            "Supplier updated",
        ))
    }

    fn delete_supplier(&mut self, payload: &CommandPayload) -> Result<Action, String> {
        require_permission(payload.actor.role, "supplier.create_update_delete")?;
        let input: DeleteById = parse_data(payload)?;
        let index = self
            .suppliers
            .iter()
            .position(|supplier| supplier.id == input.id)
            .ok_or_else(|| "supplier_not_found".to_string())?;
        let supplier = self.suppliers.remove(index);
        Ok(action(
            "supplier.create_update_delete",
            "supplier",
            supplier.id.clone(),
            json_string(&supplier),
            None,
            "Supplier deleted",
        ))
    }

    fn ensure_stock_item(&mut self, reference: &str, source_id: &str, now: u64) {
        if self
            .stock_items
            .iter()
            .any(|item| item.reference_normalized == reference)
        {
            return;
        }
        self.stock_items.push(StockItem {
            id: format!("stock-auto-{source_id}"),
            reference: reference.to_string(),
            reference_normalized: reference.to_string(),
            quantity: 0,
            minimum_quantity: 0,
            manufacturer: None,
            location: None,
            created_at_ms: now,
            updated_at_ms: now,
        });
    }
}

type Action = (
    &'static str,
    ExecutionReceipt,
    Option<String>,
    Option<String>,
    String,
);

fn action(
    permission: &'static str,
    aggregate: &str,
    aggregate_id: String,
    before: Option<String>,
    after: Option<String>,
    description: &str,
) -> Action {
    (
        permission,
        ExecutionReceipt {
            aggregate: aggregate.to_string(),
            aggregate_id,
            event_name: format!("{aggregate}.changed"),
            replayed: false,
        },
        before,
        after,
        description.to_string(),
    )
}

fn validate_actor(actor: &Actor) -> Result<(), String> {
    require_text(&actor.id, "actor_id_required")?;
    require_text(&actor.name, "actor_name_required")
}
fn require_permission(role: Role, permission: &'static str) -> Result<(), String> {
    if can(role, permission) {
        Ok(())
    } else {
        Err("forbidden".to_string())
    }
}
fn require_text(value: &str, error: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(error.to_string())
    } else {
        Ok(())
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
fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
fn normalized_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| normalize_reference(&value))
        .filter(|value| !value.is_empty())
}
fn set_optional(target: &mut Option<String>, value: Option<String>) {
    if value.is_some() {
        *target = clean_optional(value);
    }
}
fn set_if_some(target: &mut String, value: Option<String>) {
    if let Some(value) = value {
        *target = value;
    }
}
fn json_string<T: Serialize>(value: &T) -> Option<String> {
    serde_json::to_string(value).ok()
}
fn parse_data<T: for<'de> Deserialize<'de>>(payload: &CommandPayload) -> Result<T, String> {
    serde_json::from_value(payload.data.clone()).map_err(|_| "invalid_command_data".to_string())
}
fn validate_supplier_links(email: Option<&str>, website: Option<&str>) -> Result<(), String> {
    if let Some(email) = email.map(str::trim).filter(|value| !value.is_empty()) {
        let (local, domain) = email
            .split_once('@')
            .ok_or_else(|| "supplier_email_invalid".to_string())?;
        if local.is_empty() || !domain.contains('.') || email.contains(char::is_whitespace) {
            return Err("supplier_email_invalid".to_string());
        }
    }
    if let Some(website) = website.map(str::trim).filter(|value| !value.is_empty()) {
        if website.contains(char::is_whitespace)
            || !(website.starts_with("https://") || website.starts_with("http://"))
        {
            return Err("supplier_website_invalid".to_string());
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct CreateValve {
    tag: String,
    zone: String,
    manufacturer: Option<String>,
    serial: Option<String>,
    kit_reference: Option<String>,
    seat: Option<String>,
    dn: Option<String>,
    valve_type: Option<String>,
    actuator: Option<String>,
    manufactured_at: Option<String>,
    last_kit_changed_at: Option<String>,
    last_maintenance_at: Option<String>,
}
#[derive(Deserialize)]
struct UpdateValve {
    id: String,
    tag: Option<String>,
    zone: Option<String>,
    manufacturer: Option<String>,
    serial: Option<String>,
    kit_reference: Option<String>,
    seat: Option<String>,
    dn: Option<String>,
    valve_type: Option<String>,
    actuator: Option<String>,
    manufactured_at: Option<String>,
}
#[derive(Deserialize)]
struct AddValvePhoto {
    valve_id: String,
    blob_ref: String,
}
#[derive(Deserialize)]
struct DeleteById {
    id: String,
}
#[derive(Deserialize)]
struct RegisterMaintenance {
    valve_id: String,
    performed_at: String,
    technician: String,
    maintenance_type: MaintenanceType,
    service: String,
    notes: Option<String>,
    signature_ref: Option<String>,
    #[serde(default)]
    kit_changed: bool,
}
#[derive(Deserialize)]
struct CreateServiceOrder {
    zone: String,
    valve_id: Option<String>,
    description: String,
    priority: ServiceOrderPriority,
    technician: Option<String>,
    scheduled_for: Option<String>,
}
#[derive(Deserialize)]
struct ChangeServiceOrderStatus {
    id: String,
    status: ServiceOrderStatus,
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
#[derive(Deserialize)]
struct SupplierInput {
    name: String,
    contact: String,
    email: Option<String>,
    website: Option<String>,
    notes: Option<String>,
}
#[derive(Deserialize)]
struct UpdateSupplier {
    id: String,
    name: String,
    contact: String,
    email: Option<String>,
    website: Option<String>,
    notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command(actor_role: Role, data: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(
            &serde_json::json!({"actor":{"id":"u1","name":"Test","role":actor_role},"data":data}),
        )
        .unwrap()
    }

    #[test]
    fn create_valve_normalizes_tag_and_ensures_kit_once() {
        let mut state = ApplicationState::default();
        let payload = command(
            Role::Chefe,
            serde_json::json!({"tag":" fv  10 ","zone":"A","kit_reference":" kit-1 "}),
        );
        state
            .execute(commands::CREATE_VALVE, "c1", "idem-1", 1, &payload)
            .unwrap();
        assert_eq!(state.valves[0].tag_normalized, "FV 10");
        assert_eq!(state.stock_items.len(), 1);
        let replay = state
            .execute(commands::CREATE_VALVE, "c2", "idem-1", 2, &payload)
            .unwrap();
        assert!(replay.replayed);
        assert_eq!(state.valves.len(), 1);
    }

    #[test]
    fn maintenance_consumes_one_kit_only_once() {
        let mut state = ApplicationState::default();
        let create = command(
            Role::Chefe,
            serde_json::json!({"tag":"V1","zone":"A","kit_reference":"K1"}),
        );
        state
            .execute(commands::CREATE_VALVE, "c1", "idem-1", 1, &create)
            .unwrap();
        state.stock_items[0].quantity = 1;
        let maintenance = command(
            Role::Tecnico,
            serde_json::json!({"valve_id":"valve-c1","performed_at":"2026-08-13","technician":"Tech","maintenance_type":"preventive","service":"Inspection","signature_ref":"signatures/test.png","kit_changed":true}),
        );
        state
            .execute(
                commands::REGISTER_MAINTENANCE,
                "c2",
                "idem-2",
                2,
                &maintenance,
            )
            .unwrap();
        state
            .execute(
                commands::REGISTER_MAINTENANCE,
                "c3",
                "idem-2",
                3,
                &maintenance,
            )
            .unwrap();
        assert_eq!(state.stock_items[0].quantity, 0);
        assert_eq!(state.maintenance_records.len(), 1);
        assert_eq!(state.stock_movements.len(), 1);
    }

    #[test]
    fn maintenance_is_recorded_when_kit_stock_is_zero() {
        let mut state = ApplicationState::default();
        let create = command(
            Role::Chefe,
            serde_json::json!({"tag":"V1","zone":"A","kit_reference":"K1"}),
        );
        state
            .execute(commands::CREATE_VALVE, "c1", "idem-1", 1, &create)
            .unwrap();
        let maintenance = command(
            Role::Tecnico,
            serde_json::json!({"valve_id":"valve-c1","performed_at":"2026-08-13","technician":"Tech","maintenance_type":"corrective","service":"Repair","signature_ref":"signatures/test.png","kit_changed":true}),
        );
        state
            .execute(
                commands::REGISTER_MAINTENANCE,
                "c2",
                "idem-2",
                2,
                &maintenance,
            )
            .unwrap();
        assert!(state.maintenance_records[0].stock_consumption_pending);
        assert_eq!(state.stock_items[0].quantity, 0);
    }

    #[test]
    fn technician_cannot_create_valve_through_application_layer() {
        let mut state = ApplicationState::default();
        let payload = command(Role::Tecnico, serde_json::json!({"tag":"V1","zone":"A"}));
        assert_eq!(
            state.execute(commands::CREATE_VALVE, "c1", "idem-1", 1, &payload),
            Err("forbidden".to_string())
        );
        assert!(state.valves.is_empty());
        assert!(state.audit_events.is_empty());
    }

    #[test]
    fn completed_order_cannot_be_deleted_but_pending_order_can() {
        let mut state = ApplicationState::default();
        let create = command(
            Role::Chefe,
            serde_json::json!({"zone":"A","description":"Inspect","priority":"normal"}),
        );
        state
            .execute(commands::CREATE_SERVICE_ORDER, "c1", "idem-1", 1, &create)
            .unwrap();
        let delete = command(Role::Chefe, serde_json::json!({"id":"order-c1"}));
        state
            .execute(commands::DELETE_SERVICE_ORDER, "c2", "idem-2", 2, &delete)
            .unwrap();
        assert!(state.service_orders.is_empty());
        assert_eq!(
            state.audit_events.last().unwrap().operation,
            commands::DELETE_SERVICE_ORDER
        );

        state
            .execute(commands::CREATE_SERVICE_ORDER, "c3", "idem-3", 3, &create)
            .unwrap();
        state.service_orders[0].status = ServiceOrderStatus::Completed;
        let delete_completed = command(Role::Chefe, serde_json::json!({"id":"order-c3"}));
        assert_eq!(
            state.execute(
                commands::DELETE_SERVICE_ORDER,
                "c4",
                "idem-4",
                4,
                &delete_completed,
            ),
            Err("completed_order_cannot_be_deleted".to_string())
        );
    }

    #[test]
    fn valve_photo_is_associated_by_immutable_valve_id() {
        let mut state = ApplicationState::default();
        let create = command(Role::Chefe, serde_json::json!({"tag":"V1","zone":"A"}));
        state
            .execute(commands::CREATE_VALVE, "c1", "idem-1", 1, &create)
            .unwrap();
        let add = command(
            Role::Admin,
            serde_json::json!({"valve_id":"valve-c1","blob_ref":"valves/asset.png"}),
        );
        state
            .execute(commands::ADD_VALVE_PHOTO, "c2", "idem-2", 2, &add)
            .unwrap();
        assert_eq!(state.valve_photos[0].valve_id, "valve-c1");
        let remove = command(
            Role::Admin,
            serde_json::json!({"id":state.valve_photos[0].id}),
        );
        state
            .execute(commands::DELETE_VALVE_PHOTO, "c3", "idem-3", 3, &remove)
            .unwrap();
        assert!(state.valve_photos.is_empty());
    }

    #[test]
    fn supplier_rejects_invalid_email_and_website() {
        let mut state = ApplicationState::default();
        let invalid_email = command(
            Role::Admin,
            serde_json::json!({"name":"Supplier","contact":"Person","email":"invalid"}),
        );
        assert_eq!(
            state.execute(commands::CREATE_SUPPLIER, "c1", "idem-1", 1, &invalid_email,),
            Err("supplier_email_invalid".to_string())
        );
        let invalid_website = command(
            Role::Admin,
            serde_json::json!({"name":"Supplier","contact":"Person","website":"example.com"}),
        );
        assert_eq!(
            state.execute(
                commands::CREATE_SUPPLIER,
                "c2",
                "idem-2",
                2,
                &invalid_website,
            ),
            Err("supplier_website_invalid".to_string())
        );
        assert!(state.suppliers.is_empty());
    }
}
