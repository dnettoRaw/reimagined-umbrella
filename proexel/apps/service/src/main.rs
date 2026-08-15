use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use appcore_bin::application::{
    run_application, ApiRequest, ApiResponse, ApiRouter, Application, CommandBus, CommandEnvelope,
    CommandHandler, CommandName, CommandRegistry, CommandResult, EventEnvelope, EventName,
    EventRegistry, QueryEndpoint, QueryName, RuntimeContext, RuntimeResult,
};
use proexel_application::{can, commands, ApplicationState, Role};
use proexel_domain::{
    ComplexityLevel, OperationalStatus, PhotoOwnerType, ServiceOrderStatus, ServiceOrderTaskStatus,
    UserAccount,
};
use proexel_infrastructure::JsonFileStore;
use serde::Deserialize;
use serde_json::{json, Value};

const EVENTS: &[&str] = &[
    "item_category.created",
    "item_category.updated",
    "maintenance_guide.updated",
    "machine.created",
    "machine.updated",
    "machine_item.created",
    "machine_item.updated",
    "machine_item.removed",
    "machine_item.replaced",
    "photo.added",
    "photo.removed",
    "service_order.created",
    "service_order.started",
    "service_order.updated",
    "service_order.deleted",
    "service_order.completed",
    "service_order_task.started",
    "service_order_task.completed",
    "inspection.started",
    "inspection.completed",
    "item.status_changed",
    "restock_request.changed",
    "stock_item.changed",
    "supplier.changed",
    "user_account.changed",
];

#[derive(Clone)]
struct ProexelApplication {
    store: JsonFileStore,
}

impl Application for ProexelApplication {
    fn register_commands(&self, registry: &mut CommandRegistry) -> RuntimeResult<()> {
        for command in commands::COMMANDS {
            registry.register(CommandName::new(*command)?)?;
        }
        Ok(())
    }

    fn register_events(&self, registry: &mut EventRegistry) -> RuntimeResult<()> {
        for event in EVENTS {
            registry.register(EventName::new(*event)?)?;
        }
        Ok(())
    }

    fn register_handlers(&self, bus: &mut CommandBus) -> RuntimeResult<()> {
        for command in commands::COMMANDS {
            bus.register_handler(ProexelCommandHandler {
                command: CommandName::new(*command)?,
                store: self.store.clone(),
            })?;
        }
        Ok(())
    }

    fn register_queries(&self, router: &mut ApiRouter) -> RuntimeResult<()> {
        for query in commands::QUERIES {
            router.register_query(ProexelQuery {
                name: QueryName::new(*query)?,
                store: self.store.clone(),
            })?;
        }
        Ok(())
    }
}

struct ProexelCommandHandler {
    command: CommandName,
    store: JsonFileStore,
}

impl CommandHandler for ProexelCommandHandler {
    fn command_name(&self) -> CommandName {
        self.command.clone()
    }

    fn handle(
        &self,
        command: &CommandEnvelope,
        _context: &dyn RuntimeContext,
    ) -> RuntimeResult<CommandResult> {
        let Some(idempotency_key) = command.idempotency_key.as_deref() else {
            return Ok(CommandResult::rejected("idempotency_key_required"));
        };
        let result = self.store.transact(|state| {
            state.execute(
                command.command_name.as_str(),
                &command.command_id,
                idempotency_key,
                command.issued_at_ms,
                command.payload(),
            )
        });
        let receipt = match result {
            Ok(receipt) => receipt,
            Err(error) => {
                eprintln!(
                    "proexel command rejected command={} command_id={} reason={}",
                    command.command_name.as_str(),
                    command.command_id,
                    error
                );
                return Ok(CommandResult::rejected(error));
            }
        };
        let payload = serde_json::to_vec(&receipt).unwrap_or_default();
        let event = EventEnvelope::new(
            EventName::new(receipt.event_name)?,
            format!("evt-{}", command.command_id),
            command.app_id.clone(),
            command.node_id.clone(),
            command.issued_at_ms,
            payload,
        )?;
        Ok(CommandResult::accepted(vec![event]))
    }
}

struct ProexelQuery {
    name: QueryName,
    store: JsonFileStore,
}

impl QueryEndpoint for ProexelQuery {
    fn query_name(&self) -> &QueryName {
        &self.name
    }

    fn handle_query(&self, request: ApiRequest) -> RuntimeResult<ApiResponse> {
        let state = match self.store.read() {
            Ok(state) => state,
            Err(error) => return json_response(503, json!({"error": error})),
        };
        let envelope: Value =
            serde_json::from_slice(&request.payload).unwrap_or_else(|_| json!({}));
        let role = envelope
            .pointer("/actor/role")
            .cloned()
            .and_then(|value| serde_json::from_value::<Role>(value).ok());
        let identity_query = self.name.as_str() == commands::RESOLVE_IDENTITY;
        let permission = query_permission(self.name.as_str());
        if !identity_query
            && (role.is_none()
                || permission.is_some_and(|permission| !can(role.unwrap(), permission)))
        {
            return json_response(403, json!({"error": "forbidden"}));
        }
        let filters = envelope.get("data").cloned().unwrap_or_else(|| json!({}));
        let payload = match self.name.as_str() {
            commands::GET_OVERVIEW => overview_payload(&state),
            commands::LIST_ITEM_CATEGORIES => categories_payload(&state, &filters),
            commands::LIST_MACHINES => machines_payload(&state, &filters),
            commands::LIST_SERVICE_ORDERS => orders_payload(&state, &filters),
            commands::LIST_INSPECTIONS => inspections_payload(&state, &filters),
            commands::LIST_RESTOCK_REQUESTS => {
                json!({"items": state.restock_requests, "schema_version": state.schema_version})
            }
            commands::LIST_STOCK => {
                json!({"items": state.stock_items, "movements": state.stock_movements, "schema_version": state.schema_version})
            }
            commands::LIST_SUPPLIERS => {
                json!({"items": state.suppliers, "schema_version": state.schema_version})
            }
            commands::LIST_AUDIT => audit_payload(&state, &filters),
            commands::GET_REPORTS => reports_payload(&state),
            commands::LIST_USERS => users_payload(&state),
            commands::LIST_OPERATORS => operators_payload(&state),
            commands::RESOLVE_IDENTITY => identity_payload(&state, &filters),
            _ => json!({"error": "unknown_query"}),
        };
        json_response(200, payload)
    }
}

fn query_permission(name: &str) -> Option<&'static str> {
    match name {
        commands::GET_OVERVIEW => None,
        commands::LIST_ITEM_CATEGORIES => Some("item_category.read"),
        commands::LIST_MACHINES => Some("machine.read"),
        commands::LIST_SERVICE_ORDERS => Some("order.read"),
        commands::LIST_INSPECTIONS => Some("inspection.read"),
        commands::LIST_RESTOCK_REQUESTS => Some("restock.read"),
        commands::LIST_STOCK => Some("stock.read"),
        commands::LIST_SUPPLIERS => Some("supplier.read"),
        commands::LIST_AUDIT => Some("audit.read"),
        commands::GET_REPORTS => Some("report.read"),
        commands::LIST_USERS => Some("admin.users.manage"),
        commands::LIST_OPERATORS => Some("operator.read"),
        commands::RESOLVE_IDENTITY => None,
        _ => Some("unknown"),
    }
}

fn categories_payload(state: &ApplicationState, filters: &Value) -> Value {
    let search = normalized_filter(filters, "search");
    let active = filters.get("active").and_then(Value::as_bool);
    let mut items = state
        .item_categories
        .iter()
        .filter(|category| {
            search.is_empty()
                || category.code_normalized.contains(&search)
                || category.name.to_uppercase().contains(&search)
        })
        .filter(|category| active.is_none_or(|active| category.active == active))
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.name.cmp(&right.name));
    let items = items
        .into_iter()
        .map(|category| {
            let step_ids = category
                .maintenance_guide
                .steps
                .iter()
                .map(|step| step.id.as_str())
                .collect::<BTreeSet<_>>();
            let mut value = serde_json::to_value(category).unwrap_or_else(|_| json!({}));
            value["guide_photos"] = json!(state
                .photos
                .iter()
                .filter(|photo| {
                    photo.owner_type == PhotoOwnerType::GuideStep
                        && step_ids.contains(photo.owner_id.as_str())
                })
                .collect::<Vec<_>>());
            value
        })
        .collect::<Vec<_>>();
    json!({"items": items, "total": items.len(), "schema_version": state.schema_version})
}

fn machines_payload(state: &ApplicationState, filters: &Value) -> Value {
    let id = text_filter(filters, "id");
    let search = normalized_filter(filters, "search");
    let zone = text_filter(filters, "zone");
    let status = text_filter(filters, "status");
    let page = page_filter(filters, "page", 1, usize::MAX);
    let page_size = page_filter(filters, "page_size", 25, 500);
    let include_removed = filters
        .get("include_removed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut machines = state
        .machines
        .iter()
        .filter(|machine| id.is_empty() || machine.id == id)
        .filter(|machine| {
            search.is_empty()
                || machine.code_normalized.contains(&search)
                || machine.name.to_uppercase().contains(&search)
                || machine.zone.to_uppercase().contains(&search)
        })
        .filter(|machine| zone.is_empty() || machine.zone == zone)
        .filter(|machine| status.is_empty() || operational_status_name(machine.status) == status)
        .collect::<Vec<_>>();
    machines.sort_by(|left, right| {
        left.zone
            .cmp(&right.zone)
            .then(left.code_normalized.cmp(&right.code_normalized))
    });
    let total = machines.len();
    let start = page.saturating_sub(1).saturating_mul(page_size).min(total);
    let items = machines
        .into_iter()
        .skip(start)
        .take(page_size)
        .map(|machine| {
            let mut value = serde_json::to_value(machine).unwrap_or_else(|_| json!({}));
            let machine_items = state
                .machine_items
                .iter()
                .filter(|item| item.machine_id == machine.id && (include_removed || item.active))
                .map(|item| {
                    let mut item_value = serde_json::to_value(item).unwrap_or_else(|_| json!({}));
                    item_value["category"] = json!(state
                        .item_categories
                        .iter()
                        .find(|category| category.id == item.category_id));
                    item_value["photos"] = json!(state
                        .photos
                        .iter()
                        .filter(|photo| photo.owner_type == PhotoOwnerType::MachineItem
                            && photo.owner_id == item.id)
                        .collect::<Vec<_>>());
                    item_value["replacement_history"] = json!(state
                        .machine_item_replacements
                        .iter()
                        .filter(|replacement| replacement.machine_item_id == item.id)
                        .map(|replacement| {
                            let mut replacement_value =
                                serde_json::to_value(replacement).unwrap_or_else(|_| json!({}));
                            replacement_value["photos"] = json!(state
                                .photos
                                .iter()
                                .filter(|photo| {
                                    photo.owner_type == PhotoOwnerType::Replacement
                                        && photo.owner_id == replacement.id
                                })
                                .collect::<Vec<_>>());
                            replacement_value
                        })
                        .collect::<Vec<_>>());
                    item_value
                })
                .collect::<Vec<_>>();
            value["items"] = json!(machine_items);
            value["photos"] = json!(state
                .photos
                .iter()
                .filter(|photo| photo.owner_type == PhotoOwnerType::Machine
                    && photo.owner_id == machine.id)
                .collect::<Vec<_>>());
            value
        })
        .collect::<Vec<_>>();
    let zones = state
        .machines
        .iter()
        .map(|machine| machine.zone.clone())
        .collect::<BTreeSet<_>>();
    json!({
        "items": items,
        "total": total,
        "page": page,
        "page_size": page_size,
        "facets": {"zones": zones},
        "schema_version": state.schema_version,
    })
}

fn orders_payload(state: &ApplicationState, filters: &Value) -> Value {
    let id = text_filter(filters, "id");
    let machine_id = text_filter(filters, "machine_id");
    let status = text_filter(filters, "status");
    let operator_id = text_filter(filters, "operator_id");
    let mut orders = state
        .service_orders
        .iter()
        .filter(|order| id.is_empty() || order.id == id)
        .filter(|order| machine_id.is_empty() || order.machine_id == machine_id)
        .filter(|order| status.is_empty() || order_status_name(order.status) == status)
        .filter(|order| {
            operator_id.is_empty()
                || order
                    .tasks
                    .iter()
                    .any(|task| task.assigned_operator_id.as_deref() == Some(operator_id.as_str()))
        })
        .collect::<Vec<_>>();
    orders.sort_by_key(|order| std::cmp::Reverse(order.created_at_ms));
    let items = orders
        .into_iter()
        .map(|order| {
            let mut value = serde_json::to_value(order).unwrap_or_else(|_| json!({}));
            value["maximum_complexity_level"] = json!(order
                .tasks
                .iter()
                .map(|task| task.complexity_snapshot.get())
                .max()
                .unwrap_or(1));
            value["completed_tasks"] = json!(order
                .tasks
                .iter()
                .filter(|task| task.status == ServiceOrderTaskStatus::Completed)
                .count());
            value
        })
        .collect::<Vec<_>>();
    json!({"total": items.len(), "items": items, "schema_version": state.schema_version})
}

fn inspections_payload(state: &ApplicationState, filters: &Value) -> Value {
    let id = text_filter(filters, "id");
    let order_id = text_filter(filters, "service_order_id");
    let machine_id = text_filter(filters, "machine_id");
    let machine_item_id = text_filter(filters, "machine_item_id");
    let operator_id = text_filter(filters, "operator_id");
    let mut items = state
        .inspections
        .iter()
        .filter(|inspection| id.is_empty() || inspection.id == id)
        .filter(|inspection| {
            order_id.is_empty() || inspection.service_order_id.as_deref() == Some(order_id.as_str())
        })
        .filter(|inspection| machine_id.is_empty() || inspection.machine_id == machine_id)
        .filter(|inspection| {
            machine_item_id.is_empty() || inspection.machine_item_id == machine_item_id
        })
        .filter(|inspection| operator_id.is_empty() || inspection.operator_id == operator_id)
        .collect::<Vec<_>>();
    items.sort_by_key(|inspection| std::cmp::Reverse(inspection.started_at_ms));
    let items = items
        .into_iter()
        .map(|inspection| {
            let mut value = serde_json::to_value(inspection).unwrap_or_else(|_| json!({}));
            value["photos"] = json!(state
                .photos
                .iter()
                .filter(|photo| {
                    photo.owner_type == PhotoOwnerType::Inspection
                        && photo.owner_id == inspection.id
                })
                .collect::<Vec<_>>());
            value
        })
        .collect::<Vec<_>>();
    json!({"total": items.len(), "items": items, "schema_version": state.schema_version})
}

fn audit_payload(state: &ApplicationState, filters: &Value) -> Value {
    let search = text_filter(filters, "search").to_lowercase();
    let operation = text_filter(filters, "operation");
    let actor = text_filter(filters, "actor");
    let aggregate = text_filter(filters, "aggregate");
    let from_ms = filters.get("from_ms").and_then(Value::as_u64).unwrap_or(0);
    let to_ms = filters
        .get("to_ms")
        .and_then(Value::as_u64)
        .unwrap_or(u64::MAX);
    let page = page_filter(filters, "page", 1, usize::MAX);
    let page_size = page_filter(filters, "page_size", 50, 200);
    let events = state
        .audit_events
        .iter()
        .rev()
        .filter(|event| {
            search.is_empty()
                || event.actor.to_lowercase().contains(&search)
                || event.operation.to_lowercase().contains(&search)
                || event.aggregate_id.to_lowercase().contains(&search)
                || event
                    .description
                    .as_deref()
                    .is_some_and(|value| value.to_lowercase().contains(&search))
        })
        .filter(|event| operation.is_empty() || event.operation == operation)
        .filter(|event| actor.is_empty() || event.actor == actor)
        .filter(|event| aggregate.is_empty() || event.aggregate == aggregate)
        .filter(|event| event.created_at_ms >= from_ms && event.created_at_ms <= to_ms)
        .collect::<Vec<_>>();
    let total = events.len();
    let start = page.saturating_sub(1).saturating_mul(page_size).min(total);
    let items = events
        .into_iter()
        .skip(start)
        .take(page_size)
        .collect::<Vec<_>>();
    let operations = state
        .audit_events
        .iter()
        .map(|event| event.operation.clone())
        .collect::<BTreeSet<_>>();
    let actors = state
        .audit_events
        .iter()
        .map(|event| event.actor.clone())
        .collect::<BTreeSet<_>>();
    let aggregates = state
        .audit_events
        .iter()
        .map(|event| event.aggregate.clone())
        .collect::<BTreeSet<_>>();
    json!({
        "items": items,
        "schema_version": state.schema_version,
        "total": total,
        "page": page,
        "page_size": page_size,
        "operations": operations,
        "actors": actors,
        "aggregates": aggregates,
    })
}

fn overview_payload(state: &ApplicationState) -> Value {
    let machine_statuses = status_counts(state.machines.iter().map(|machine| machine.status));
    let active_items = state.machine_items.iter().filter(|item| item.active);
    let item_statuses = status_counts(active_items.clone().map(|item| item.status));
    let order_counts = order_status_counts(state);
    let low_stock = state
        .stock_items
        .iter()
        .filter(|item| item.quantity <= item.minimum_quantity)
        .count();
    let mut recent_inspections = state.inspections.iter().collect::<Vec<_>>();
    recent_inspections.sort_by_key(|inspection| std::cmp::Reverse(inspection.started_at_ms));
    let mut upcoming_orders = state
        .service_orders
        .iter()
        .filter(|order| {
            order.scheduled_for.is_some()
                && !matches!(
                    order.status,
                    ServiceOrderStatus::Completed | ServiceOrderStatus::Cancelled
                )
        })
        .collect::<Vec<_>>();
    upcoming_orders.sort_by(|left, right| left.scheduled_for.cmp(&right.scheduled_for));
    json!({
        "schema_version": state.schema_version,
        "machines": {"total": state.machines.len(), "by_status": machine_statuses},
        "machine_items": {"total": active_items.count(), "by_status": item_statuses},
        "orders": order_counts,
        "stock": {"low": low_stock, "total": state.stock_items.len()},
        "recent_inspections": recent_inspections.into_iter().take(5).collect::<Vec<_>>(),
        "upcoming_orders": upcoming_orders.into_iter().take(5).collect::<Vec<_>>(),
    })
}

fn reports_payload(state: &ApplicationState) -> Value {
    let overview = overview_payload(state);
    let mut by_zone = BTreeMap::<String, (usize, usize, usize)>::new();
    for machine in &state.machines {
        let row = by_zone.entry(machine.zone.clone()).or_default();
        row.0 += 1;
        for item in state
            .machine_items
            .iter()
            .filter(|item| item.active && item.machine_id == machine.id)
        {
            row.1 += 1;
            if matches!(
                item.status,
                OperationalStatus::Critical | OperationalStatus::MaintenanceRequired
            ) {
                row.2 += 1;
            }
        }
    }
    let zones = by_zone
        .into_iter()
        .map(|(zone, (machines, items, critical_items))| {
            json!({
                "zone": zone,
                "machines": machines,
                "items": items,
                "critical_items": critical_items,
            })
        })
        .collect::<Vec<_>>();
    let critical_items = state
        .machine_items
        .iter()
        .filter(|item| {
            item.active
                && matches!(
                    item.status,
                    OperationalStatus::Critical | OperationalStatus::MaintenanceRequired
                )
        })
        .map(|item| {
            json!({
                "item": item,
                "machine": state.machines.iter().find(|machine| machine.id == item.machine_id),
                "category": state.item_categories.iter().find(|category| category.id == item.category_id),
            })
        })
        .collect::<Vec<_>>();
    let mut recent_inspections = state.inspections.iter().collect::<Vec<_>>();
    recent_inspections.sort_by_key(|inspection| std::cmp::Reverse(inspection.started_at_ms));
    json!({
        "schema_version": state.schema_version,
        "generated_at_ms": now_ms(),
        "overview": overview,
        "by_zone": zones,
        "critical_items": critical_items,
        "recent_inspections": recent_inspections,
    })
}

fn users_payload(state: &ApplicationState) -> Value {
    let items = state
        .user_accounts
        .iter()
        .map(|user| {
            json!({
                "id": user.id,
                "email": user.email,
                "name": user.name,
                "role": user.role,
                "active": user.active,
                "maximum_repair_level": user.maximum_repair_level,
                "has_pin": user.pin_hash.is_some(),
                "auth_version": user.auth_version,
                "created_at_ms": user.created_at_ms,
                "updated_at_ms": user.updated_at_ms,
            })
        })
        .collect::<Vec<_>>();
    json!({"items": items, "schema_version": state.schema_version})
}

fn operators_payload(state: &ApplicationState) -> Value {
    let items = state
        .user_accounts
        .iter()
        .filter(|user| user.active && matches!(user.role.as_str(), "admin" | "chefe" | "tecnico"))
        .map(|user| {
            json!({
                "id": user.id,
                "name": user.name,
                "role": user.role,
                "active": user.active,
                "maximum_repair_level": user.maximum_repair_level,
            })
        })
        .collect::<Vec<_>>();
    json!({"items": items, "schema_version": state.schema_version})
}

fn identity_payload(state: &ApplicationState, filters: &Value) -> Value {
    let email = filters
        .get("email")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_lowercase());
    let id = filters.get("id").and_then(Value::as_str).map(str::trim);
    let user = state.user_accounts.iter().find(|user| {
        email.as_deref().is_some_and(|email| user.email == email)
            || id.is_some_and(|id| user.id == id)
    });
    json!({"user": user})
}

fn status_counts(
    statuses: impl Iterator<Item = OperationalStatus>,
) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for status in statuses {
        *counts.entry(operational_status_name(status)).or_default() += 1;
    }
    counts
}

fn order_status_counts(state: &ApplicationState) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for status in ["pending", "in_progress", "completed", "cancelled"] {
        counts.insert(status, 0);
    }
    for order in &state.service_orders {
        *counts.entry(order_status_name(order.status)).or_default() += 1;
    }
    counts
}

fn operational_status_name(status: OperationalStatus) -> &'static str {
    match status {
        OperationalStatus::Unknown => "unknown",
        OperationalStatus::Ok => "ok",
        OperationalStatus::Attention => "attention",
        OperationalStatus::Critical => "critical",
        OperationalStatus::MaintenanceRequired => "maintenance_required",
        OperationalStatus::UnderMaintenance => "under_maintenance",
        OperationalStatus::Disabled => "disabled",
    }
}

fn order_status_name(status: ServiceOrderStatus) -> &'static str {
    match status {
        ServiceOrderStatus::Pending => "pending",
        ServiceOrderStatus::InProgress => "in_progress",
        ServiceOrderStatus::Completed => "completed",
        ServiceOrderStatus::Cancelled => "cancelled",
    }
}

fn text_filter(filters: &Value, key: &str) -> String {
    filters
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

fn normalized_filter(filters: &Value, key: &str) -> String {
    text_filter(filters, key).to_uppercase()
}

fn page_filter(filters: &Value, key: &str, default: usize, maximum: usize) -> usize {
    filters
        .get(key)
        .and_then(Value::as_u64)
        .unwrap_or(default as u64)
        .max(1)
        .min(maximum as u64) as usize
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn json_response(status_code: u16, value: Value) -> RuntimeResult<ApiResponse> {
    Ok(ApiResponse {
        status_code,
        payload: serde_json::to_vec(&value).unwrap_or_default(),
    })
}

fn state_path() -> PathBuf {
    if let Some(path) = std::env::var_os("PROEXEL_DATA_FILE") {
        return PathBuf::from(path);
    }
    let manifest = std::env::var_os("APPCORE_DEPLOYMENT_MANIFEST")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("proexel/apps/service/deployment.local.toml"));
    manifest
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        // The existing path is retained so schema migration can happen in place.
        .join("target/runtime/storage/proexel-state-v1.json")
}

#[derive(Deserialize)]
struct SeedUser {
    id: String,
    email: String,
    name: String,
    role: String,
    password_hash: String,
    #[serde(default)]
    pin_hash: Option<String>,
    #[serde(default)]
    maximum_repair_level: Option<ComplexityLevel>,
    #[serde(default = "default_true")]
    active: bool,
}

fn default_true() -> bool {
    true
}

fn default_repair_level(role: &str) -> ComplexityLevel {
    let value = if matches!(role, "admin" | "chefe") {
        5
    } else {
        3
    };
    ComplexityLevel::new(value).expect("default repair levels are valid")
}

fn seed_users_from_environment(store: &JsonFileStore) -> Result<(), String> {
    let raw = std::env::var("PROEXEL_AUTH_USERS").unwrap_or_else(|_| "[]".to_string());
    let seeds: Vec<SeedUser> =
        serde_json::from_str(&raw).map_err(|_| "auth_users_invalid".to_string())?;
    if seeds.is_empty() {
        return Ok(());
    }
    store.transact(|state| {
        state.seed_users(
            seeds
                .into_iter()
                .map(|seed| {
                    let maximum_repair_level = seed
                        .maximum_repair_level
                        .unwrap_or_else(|| default_repair_level(&seed.role));
                    UserAccount {
                        id: seed.id,
                        email: seed.email,
                        name: seed.name,
                        role: seed.role,
                        password_hash: seed.password_hash,
                        pin_hash: seed.pin_hash,
                        active: seed.active,
                        maximum_repair_level,
                        auth_version: 1,
                        created_at_ms: 0,
                        updated_at_ms: 0,
                    }
                })
                .collect(),
        )
    })
}

fn main() {
    let store = match JsonFileStore::new(state_path()) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("proexel storage failed: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = seed_users_from_environment(&store) {
        eprintln!("proexel identity seed failed: {error}");
        std::process::exit(1);
    }
    if let Err(error) = run_application(&ProexelApplication { store }) {
        eprintln!("proexel service failed: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proexel_domain::{Machine, MachineItem, ReplacementSpecification, ServiceOrderPriority};

    fn machine(id: &str, zone: &str, status: OperationalStatus) -> Machine {
        Machine {
            id: id.to_string(),
            code: id.to_uppercase(),
            code_normalized: id.to_uppercase(),
            name: format!("Machine {id}"),
            description: None,
            zone: zone.to_string(),
            location: None,
            manufacturer: None,
            model: None,
            serial_number: None,
            status,
            main_photo_id: None,
            active: true,
            created_at_ms: 0,
            updated_at_ms: 0,
        }
    }

    fn item(index: usize, machine_id: &str, status: OperationalStatus) -> MachineItem {
        MachineItem {
            id: format!("item-{index}"),
            machine_id: machine_id.to_string(),
            category_id: "category-1".to_string(),
            name: format!("Item {index}"),
            code: format!("I-{index:05}"),
            code_normalized: format!("I-{index:05}"),
            complexity_level: ComplexityLevel::new(3).unwrap(),
            status,
            position: index as u32,
            location_description: None,
            custom_field_values: BTreeMap::new(),
            installed_component: None,
            replacement_specification: ReplacementSpecification::default(),
            notes: None,
            active: true,
            removed_at_ms: None,
            created_at_ms: index as u64,
            updated_at_ms: index as u64,
        }
    }

    #[test]
    fn reports_group_machine_items_by_zone_and_status() {
        let mut state = ApplicationState::default();
        state
            .machines
            .push(machine("machine-1", "Line 1", OperationalStatus::Critical));
        state
            .machine_items
            .push(item(1, "machine-1", OperationalStatus::Critical));

        let report = reports_payload(&state);

        assert_eq!(report["overview"]["machines"]["by_status"]["critical"], 1);
        assert_eq!(report["by_zone"][0]["zone"], "Line 1");
        assert_eq!(report["by_zone"][0]["critical_items"], 1);
        assert_eq!(report["critical_items"][0]["item"]["code"], "I-00001");
    }

    #[test]
    fn query_permissions_cover_every_registered_query() {
        for query in commands::QUERIES {
            assert_ne!(
                query_permission(query),
                Some("unknown"),
                "missing RBAC mapping for {query}"
            );
        }
        assert!(can(
            Role::Tecnico,
            query_permission(commands::LIST_MACHINES).unwrap()
        ));
        assert!(!can(
            Role::Compras,
            query_permission(commands::LIST_MACHINES).unwrap()
        ));
        assert!(can(
            Role::Chefe,
            query_permission(commands::GET_REPORTS).unwrap()
        ));
    }

    #[test]
    fn administrative_user_list_never_exposes_credential_hashes() {
        let mut state = ApplicationState::default();
        state.user_accounts.push(UserAccount {
            id: "u1".to_string(),
            email: "admin@example.com".to_string(),
            name: "Admin".to_string(),
            role: "admin".to_string(),
            password_hash: "scrypt$salt$secret".to_string(),
            pin_hash: Some("scrypt$pin$secret".to_string()),
            active: true,
            maximum_repair_level: ComplexityLevel::new(5).unwrap(),
            auth_version: 3,
            created_at_ms: 1,
            updated_at_ms: 2,
        });

        let payload = users_payload(&state);
        let encoded = payload.to_string();
        assert!(!encoded.contains("password_hash"));
        assert!(!encoded.contains("pin_hash"));
        assert!(!encoded.contains("scrypt$"));
        assert_eq!(payload["items"][0]["has_pin"], true);
        assert_eq!(payload["items"][0]["maximum_repair_level"], 5);
    }

    #[test]
    fn production_volume_machine_query_is_paginated_and_bounded() {
        let started = std::time::Instant::now();
        let mut state = ApplicationState::default();
        for index in 0..2_000 {
            let machine_id = format!("machine-{index}");
            state.machines.push(machine(
                &machine_id,
                &format!("Zone {}", index % 20),
                OperationalStatus::Ok,
            ));
            state
                .machine_items
                .push(item(index, &machine_id, OperationalStatus::Ok));
        }
        let machines = machines_payload(&state, &json!({"page": 20, "page_size": 50}));
        assert_eq!(machines["total"], 2_000);
        assert_eq!(machines["items"].as_array().unwrap().len(), 50);
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
    }

    #[test]
    fn order_status_count_includes_cancelled_and_empty_buckets() {
        let state = ApplicationState::default();
        let counts = order_status_counts(&state);
        assert_eq!(counts["pending"], 0);
        assert_eq!(counts["cancelled"], 0);
        let _ = ServiceOrderPriority::Normal;
        let _ = proexel_domain::InspectionStatus::Completed;
    }
}
