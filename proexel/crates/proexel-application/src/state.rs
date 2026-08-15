use std::collections::BTreeMap;

use proexel_domain::{
    adjust_stock, AuditEvent, ComplexityLevel, ItemCategory, ItemInspection, Machine, MachineItem,
    MachineItemReplacement, PhotoAsset, RestockRequest, RestockStatus, ServiceOrder, StockItem,
    StockMovement, StockMovementKind, Supplier, UserAccount,
};
use serde::{Deserialize, Serialize};

use crate::{commands, permissions::can, Role};

pub const SCHEMA_VERSION: u32 = 2;

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
    pub item_categories: Vec<ItemCategory>,
    #[serde(default)]
    pub machines: Vec<Machine>,
    #[serde(default)]
    pub machine_items: Vec<MachineItem>,
    #[serde(default)]
    pub machine_item_replacements: Vec<MachineItemReplacement>,
    #[serde(default)]
    pub photos: Vec<PhotoAsset>,
    #[serde(default)]
    pub service_orders: Vec<ServiceOrder>,
    #[serde(default)]
    pub inspections: Vec<ItemInspection>,
    #[serde(default)]
    pub restock_requests: Vec<RestockRequest>,
    #[serde(default)]
    pub stock_items: Vec<StockItem>,
    #[serde(default)]
    pub stock_movements: Vec<StockMovement>,
    #[serde(default)]
    pub suppliers: Vec<Supplier>,
    #[serde(default)]
    pub user_accounts: Vec<UserAccount>,
    #[serde(default)]
    pub audit_events: Vec<AuditEvent>,
    #[serde(default)]
    pub processed_commands: BTreeMap<String, ExecutionReceipt>,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            item_categories: Vec::new(),
            machines: Vec::new(),
            machine_items: Vec::new(),
            machine_item_replacements: Vec::new(),
            photos: Vec::new(),
            service_orders: Vec::new(),
            inspections: Vec::new(),
            restock_requests: Vec::new(),
            stock_items: Vec::new(),
            stock_movements: Vec::new(),
            suppliers: Vec::new(),
            user_accounts: Vec::new(),
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
            commands::CREATE_ITEM_CATEGORY => {
                self.create_item_category(command_id, issued_at_ms, &payload)?
            }
            commands::UPDATE_ITEM_CATEGORY => self.update_item_category(issued_at_ms, &payload)?,
            commands::CREATE_MACHINE => self.create_machine(command_id, issued_at_ms, &payload)?,
            commands::UPDATE_MACHINE => self.update_machine(issued_at_ms, &payload)?,
            commands::ADD_MACHINE_ITEM => {
                self.add_machine_item(command_id, issued_at_ms, &payload)?
            }
            commands::UPDATE_MACHINE_ITEM => self.update_machine_item(issued_at_ms, &payload)?,
            commands::REORDER_MACHINE_ITEMS => {
                self.reorder_machine_items(issued_at_ms, &payload)?
            }
            commands::REMOVE_MACHINE_ITEM => self.remove_machine_item(issued_at_ms, &payload)?,
            commands::REPLACE_MACHINE_ITEM => {
                self.replace_machine_item(command_id, issued_at_ms, &payload)?
            }
            commands::ADD_PHOTO => self.add_photo(command_id, issued_at_ms, &payload)?,
            commands::DELETE_PHOTO => self.delete_photo(&payload)?,
            commands::CREATE_SERVICE_ORDER => {
                self.create_service_order(command_id, issued_at_ms, &payload)?
            }
            commands::START_SERVICE_ORDER => self.start_service_order(issued_at_ms, &payload)?,
            commands::ASSIGN_ORDER_TASK => self.assign_order_task(issued_at_ms, &payload)?,
            commands::DELETE_SERVICE_ORDER => self.delete_service_order(&payload)?,
            commands::COMPLETE_SERVICE_ORDER => {
                self.complete_service_order(issued_at_ms, &payload)?
            }
            commands::START_INSPECTION => {
                self.start_inspection(command_id, issued_at_ms, &payload)?
            }
            commands::COMPLETE_INSPECTION => self.complete_inspection(issued_at_ms, &payload)?,
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
            commands::CREATE_USER => self.create_user(command_id, issued_at_ms, &payload)?,
            commands::UPDATE_USER => self.update_user(issued_at_ms, &payload)?,
            commands::RESET_USER_CREDENTIALS => {
                self.reset_user_credentials(issued_at_ms, &payload)?
            }
            _ => return Err("unknown_command".to_string()),
        };

        if !can(payload.actor.role, permission) {
            return Err("forbidden".to_string());
        }

        self.schema_version = SCHEMA_VERSION;
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

    fn create_restock_request(
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

    pub fn seed_users(&mut self, mut users: Vec<UserAccount>) -> Result<(), String> {
        if !self.user_accounts.is_empty() {
            return Ok(());
        }
        let unique_ids = users
            .iter()
            .map(|user| user.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        if unique_ids.len() != users.len() {
            return Err("user_id_already_exists".to_string());
        }
        for user in &mut users {
            user.email = normalize_email(&user.email)?;
            require_text(&user.name, "user_name_required")?;
            validate_role_name(&user.role)?;
            validate_auth_hash(&user.password_hash, "password_hash_invalid")?;
            if let Some(pin_hash) = user.pin_hash.as_deref() {
                validate_auth_hash(pin_hash, "pin_hash_invalid")?;
            }
        }
        let unique_emails = users
            .iter()
            .map(|user| user.email.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        if unique_emails.len() != users.len() {
            return Err("user_email_already_exists".to_string());
        }
        if !users.iter().any(|user| user.active && user.role == "admin") {
            return Err("active_admin_required".to_string());
        }
        self.user_accounts = users;
        Ok(())
    }

    fn create_user(
        &mut self,
        command_id: &str,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "admin.users.manage")?;
        let input: CreateUser = parse_data(payload)?;
        let email = normalize_email(&input.email)?;
        require_text(&input.name, "user_name_required")?;
        validate_auth_hash(&input.password_hash, "password_hash_invalid")?;
        if let Some(pin_hash) = input.pin_hash.as_deref() {
            validate_auth_hash(pin_hash, "pin_hash_invalid")?;
        }
        if self.user_accounts.iter().any(|user| user.email == email) {
            return Err("user_email_already_exists".to_string());
        }
        let id = format!("user-{command_id}");
        let user = UserAccount {
            id: id.clone(),
            email,
            name: input.name.trim().to_string(),
            role: role_name(input.role).to_string(),
            password_hash: input.password_hash,
            pin_hash: input.pin_hash,
            active: true,
            maximum_repair_level: input.maximum_repair_level,
            auth_version: 1,
            created_at_ms: now,
            updated_at_ms: now,
        };
        let after = user_audit_value(&user);
        self.user_accounts.push(user);
        Ok(action(
            "admin.users.manage",
            "user_account",
            id,
            None,
            after,
            "User created",
        ))
    }

    fn update_user(&mut self, now: u64, payload: &CommandPayload) -> Result<Action, String> {
        require_permission(payload.actor.role, "admin.users.manage")?;
        let input: UpdateUser = parse_data(payload)?;
        let email = normalize_email(&input.email)?;
        require_text(&input.name, "user_name_required")?;
        let index = self
            .user_accounts
            .iter()
            .position(|user| user.id == input.id)
            .ok_or_else(|| "user_not_found".to_string())?;
        if self
            .user_accounts
            .iter()
            .any(|user| user.id != input.id && user.email == email)
        {
            return Err("user_email_already_exists".to_string());
        }
        let current = &self.user_accounts[index];
        let removes_active_admin = current.active
            && current.role == "admin"
            && (!input.active || input.role != Role::Admin);
        if removes_active_admin
            && self
                .user_accounts
                .iter()
                .filter(|user| user.active && user.role == "admin")
                .count()
                <= 1
        {
            return Err("last_active_admin_required".to_string());
        }
        let before = user_audit_value(current);
        let user = &mut self.user_accounts[index];
        user.email = email;
        user.name = input.name.trim().to_string();
        user.role = role_name(input.role).to_string();
        user.active = input.active;
        user.maximum_repair_level = input.maximum_repair_level;
        user.auth_version = user.auth_version.saturating_add(1);
        user.updated_at_ms = now;
        let after = user_audit_value(user);
        Ok(action(
            "admin.users.manage",
            "user_account",
            input.id,
            before,
            after,
            "User updated",
        ))
    }

    fn reset_user_credentials(
        &mut self,
        now: u64,
        payload: &CommandPayload,
    ) -> Result<Action, String> {
        require_permission(payload.actor.role, "admin.users.manage")?;
        let input: ResetUserCredentials = parse_data(payload)?;
        if input.password_hash.is_none() && input.pin_hash.is_none() && !input.clear_pin {
            return Err("credential_change_required".to_string());
        }
        if let Some(password_hash) = input.password_hash.as_deref() {
            validate_auth_hash(password_hash, "password_hash_invalid")?;
        }
        if let Some(pin_hash) = input.pin_hash.as_deref() {
            validate_auth_hash(pin_hash, "pin_hash_invalid")?;
        }
        let user = self
            .user_accounts
            .iter_mut()
            .find(|user| user.id == input.id)
            .ok_or_else(|| "user_not_found".to_string())?;
        let before = user_audit_value(user);
        if let Some(password_hash) = input.password_hash {
            user.password_hash = password_hash;
        }
        if input.clear_pin {
            user.pin_hash = None;
        } else if let Some(pin_hash) = input.pin_hash {
            user.pin_hash = Some(pin_hash);
        }
        user.auth_version = user.auth_version.saturating_add(1);
        user.updated_at_ms = now;
        let after = user_audit_value(user);
        Ok(action(
            "admin.users.manage",
            "user_account",
            input.id,
            before,
            after,
            "User credentials reset",
        ))
    }
}

pub(crate) type Action = (
    &'static str,
    ExecutionReceipt,
    Option<String>,
    Option<String>,
    String,
);

pub(crate) fn action(
    permission: &'static str,
    aggregate: &str,
    aggregate_id: String,
    before: Option<String>,
    after: Option<String>,
    description: &str,
) -> Action {
    domain_action(
        permission,
        &format!("{aggregate}.changed"),
        aggregate,
        aggregate_id,
        before,
        after,
        description,
    )
}

pub(crate) fn domain_action(
    permission: &'static str,
    event_name: &str,
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
            event_name: event_name.to_string(),
            replayed: false,
        },
        before,
        after,
        description.to_string(),
    )
}

pub(crate) fn require_permission(role: Role, permission: &'static str) -> Result<(), String> {
    if can(role, permission) {
        Ok(())
    } else {
        Err("forbidden".to_string())
    }
}

pub(crate) fn require_text(value: &str, error: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(error.to_string())
    } else {
        Ok(())
    }
}

pub(crate) fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn json_string<T: Serialize>(value: &T) -> Option<String> {
    serde_json::to_string(value).ok()
}

pub(crate) fn parse_data<T: for<'de> Deserialize<'de>>(
    payload: &CommandPayload,
) -> Result<T, String> {
    serde_json::from_value(payload.data.clone()).map_err(|_| "invalid_command_data".to_string())
}

fn validate_actor(actor: &Actor) -> Result<(), String> {
    require_text(&actor.id, "actor_id_required")?;
    require_text(&actor.name, "actor_name_required")
}

fn role_name(role: Role) -> &'static str {
    match role {
        Role::Admin => "admin",
        Role::Chefe => "chefe",
        Role::Compras => "compras",
        Role::Tecnico => "tecnico",
    }
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

fn normalize_email(value: &str) -> Result<String, String> {
    let email = value.trim().to_lowercase();
    let (local, domain) = email
        .split_once('@')
        .ok_or_else(|| "user_email_invalid".to_string())?;
    if local.is_empty() || !domain.contains('.') || email.contains(char::is_whitespace) {
        return Err("user_email_invalid".to_string());
    }
    Ok(email)
}

fn validate_role_name(role: &str) -> Result<(), String> {
    if matches!(role, "admin" | "chefe" | "compras" | "tecnico") {
        Ok(())
    } else {
        Err("user_role_invalid".to_string())
    }
}

fn validate_auth_hash(value: &str, error: &str) -> Result<(), String> {
    let mut parts = value.split('$');
    let algorithm = parts.next();
    let salt = parts.next().unwrap_or_default();
    let digest = parts.next().unwrap_or_default();
    if algorithm != Some("scrypt")
        || salt.is_empty()
        || digest.len() < 32
        || !digest.len().is_multiple_of(2)
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        || parts.next().is_some()
    {
        return Err(error.to_string());
    }
    Ok(())
}

fn user_audit_value(user: &UserAccount) -> Option<String> {
    json_string(&serde_json::json!({
        "id": user.id,
        "email": user.email,
        "name": user.name,
        "role": user.role,
        "active": user.active,
        "maximum_repair_level": user.maximum_repair_level,
        "has_pin": user.pin_hash.is_some(),
        "auth_version": user.auth_version,
    }))
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

#[derive(Deserialize)]
struct CreateUser {
    email: String,
    name: String,
    role: Role,
    password_hash: String,
    pin_hash: Option<String>,
    maximum_repair_level: ComplexityLevel,
}

#[derive(Deserialize)]
struct UpdateUser {
    id: String,
    email: String,
    name: String,
    role: Role,
    active: bool,
    maximum_repair_level: ComplexityLevel,
}

#[derive(Deserialize)]
struct ResetUserCredentials {
    id: String,
    password_hash: Option<String>,
    pin_hash: Option<String>,
    #[serde(default)]
    clear_pin: bool,
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
    fn supplier_rejects_invalid_links() {
        let mut state = ApplicationState::default();
        let invalid = command(
            Role::Admin,
            serde_json::json!({"name":"Supplier","contact":"Person","website":"example.com"}),
        );
        assert_eq!(
            state.execute(commands::CREATE_SUPPLIER, "c1", "idem-1", 1, &invalid),
            Err("supplier_website_invalid".to_string())
        );
    }

    #[test]
    fn user_management_redacts_credentials_and_persists_repair_level() {
        let mut state = ApplicationState::default();
        let create = command(
            Role::Admin,
            serde_json::json!({
                "email":"ADMIN@EXAMPLE.COM",
                "name":"Main Admin",
                "role":"admin",
                "password_hash":format!("scrypt$salt${}", "ab".repeat(32)),
                "maximum_repair_level":5
            }),
        );
        state
            .execute(commands::CREATE_USER, "u1", "user-idem-1", 1, &create)
            .unwrap();
        assert_eq!(state.user_accounts[0].maximum_repair_level.get(), 5);
        let snapshot = state
            .audit_events
            .last()
            .unwrap()
            .after_json
            .as_deref()
            .unwrap();
        assert!(!snapshot.contains("password_hash"));
        assert!(!snapshot.contains("scrypt$"));
    }

    #[test]
    fn last_active_administrator_cannot_be_disabled() {
        let mut state = ApplicationState::default();
        let create = command(
            Role::Admin,
            serde_json::json!({
                "email":"admin@example.com",
                "name":"Main Admin",
                "role":"admin",
                "password_hash":format!("scrypt$salt${}", "ab".repeat(32)),
                "maximum_repair_level":5
            }),
        );
        state
            .execute(commands::CREATE_USER, "u1", "user-idem-1", 1, &create)
            .unwrap();
        let disable = command(
            Role::Admin,
            serde_json::json!({
                "id":"user-u1","email":"admin@example.com","name":"Main Admin",
                "role":"admin","active":false,"maximum_repair_level":5
            }),
        );
        assert_eq!(
            state.execute(commands::UPDATE_USER, "u2", "user-idem-2", 2, &disable),
            Err("last_active_admin_required".to_string())
        );
    }
}
