use std::collections::BTreeMap;

use proexel_domain::{
    AuditEvent, ItemCategory, ItemInspection, Machine, MachineItem, MachineItemReplacement,
    PhotoAsset, RestockRequest, ServiceOrder, StockItem, StockMovement, Supplier, UserAccount,
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

pub(crate) fn role_name(role: Role) -> &'static str {
    match role {
        Role::Admin => "admin",
        Role::Chefe => "chefe",
        Role::Compras => "compras",
        Role::Tecnico => "tecnico",
    }
}
