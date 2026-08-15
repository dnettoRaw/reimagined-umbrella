use std::collections::BTreeMap;
use std::path::PathBuf;

use appcore_bin::application::{
    run_application, ApiRequest, ApiResponse, ApiRouter, Application, CommandBus, CommandEnvelope,
    CommandHandler, CommandName, CommandRegistry, CommandResult, EventEnvelope, EventName,
    EventRegistry, QueryEndpoint, QueryName, RuntimeContext, RuntimeResult,
};
use proexel_application::{can, commands, ApplicationState, Role};
use proexel_domain::{maintenance_health, MaintenanceHealth};
use proexel_infrastructure::JsonFileStore;
use serde_json::{json, Value};

const EVENTS: &[&str] = &[
    "valve.changed",
    "valve_photo.changed",
    "maintenance.changed",
    "service_order.changed",
    "restock_request.changed",
    "stock_item.changed",
    "supplier.changed",
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
        let permission = query_permission(self.name.as_str());
        if role.is_none() || permission.is_some_and(|permission| !can(role.unwrap(), permission)) {
            return json_response(403, json!({"error": "forbidden"}));
        }
        let filters = envelope.get("data").cloned().unwrap_or_else(|| json!({}));
        let payload = match self.name.as_str() {
            commands::GET_OVERVIEW => overview_payload(&state),
            commands::LIST_VALVES => valves_payload(&state, &filters),
            commands::LIST_MAINTENANCE => {
                json!({"items": state.maintenance_records, "schema_version": state.schema_version})
            }
            commands::LIST_SERVICE_ORDERS => {
                json!({"items": state.service_orders, "schema_version": state.schema_version})
            }
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
            _ => json!({"error": "unknown_query"}),
        };
        json_response(200, payload)
    }
}

fn audit_payload(state: &ApplicationState, filters: &Value) -> Value {
    let search = filters
        .get("search")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let operation = filters
        .get("operation")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let actor = filters
        .get("actor")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let aggregate = filters
        .get("aggregate")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let from_ms = filters.get("from_ms").and_then(Value::as_u64).unwrap_or(0);
    let to_ms = filters
        .get("to_ms")
        .and_then(Value::as_u64)
        .unwrap_or(u64::MAX);
    let page = filters
        .get("page")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .max(1) as usize;
    let page_size = filters
        .get("page_size")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .clamp(1, 200) as usize;
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
        .collect::<std::collections::BTreeSet<_>>();
    let actors = state
        .audit_events
        .iter()
        .map(|event| event.actor.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let aggregates = state
        .audit_events
        .iter()
        .map(|event| event.aggregate.clone())
        .collect::<std::collections::BTreeSet<_>>();
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

fn query_permission(name: &str) -> Option<&'static str> {
    match name {
        commands::GET_OVERVIEW => None,
        commands::LIST_VALVES => Some("valve.read"),
        commands::LIST_MAINTENANCE => Some("maintenance.read"),
        commands::LIST_SERVICE_ORDERS => Some("order.read"),
        commands::LIST_RESTOCK_REQUESTS => Some("restock.read"),
        commands::LIST_STOCK => Some("stock.read"),
        commands::LIST_SUPPLIERS => Some("supplier.read"),
        commands::LIST_AUDIT => Some("audit.read"),
        commands::GET_REPORTS => Some("report.read"),
        _ => Some("unknown"),
    }
}

fn valves_payload(state: &ApplicationState, filters: &Value) -> Value {
    let id = filters
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let search = filters
        .get("search")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_uppercase();
    let zone = filters
        .get("zone")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let health_filter = filters
        .get("health")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let valve_type = filters
        .get("valve_type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let sort = filters.get("sort").and_then(Value::as_str).unwrap_or("tag");
    let descending = filters
        .get("direction")
        .and_then(Value::as_str)
        .is_some_and(|value| value == "desc");
    let page = filters
        .get("page")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .max(1) as usize;
    let page_size = filters
        .get("page_size")
        .and_then(Value::as_u64)
        .unwrap_or(25)
        .clamp(1, 500) as usize;
    let mut valves = state
        .valves
        .iter()
        .filter(|valve| id.is_empty() || valve.id == id)
        .filter(|valve| {
            search.is_empty()
                || valve.tag_normalized.contains(&search)
                || valve.zone.to_uppercase().contains(&search)
        })
        .filter(|valve| zone.is_empty() || valve.zone == zone)
        .filter(|valve| {
            health_filter.is_empty()
                || health_name(valve.last_maintenance_at.as_deref()) == health_filter
        })
        .filter(|valve| valve_type.is_empty() || valve.valve_type.as_deref() == Some(valve_type))
        .collect::<Vec<_>>();
    valves.sort_by(|left, right| {
        let order = match sort {
            "zone" => left.zone.cmp(&right.zone).then(left.tag.cmp(&right.tag)),
            "health" => health_name(left.last_maintenance_at.as_deref())
                .cmp(health_name(right.last_maintenance_at.as_deref()))
                .then(left.tag.cmp(&right.tag)),
            "last_maintenance" => left
                .last_maintenance_at
                .cmp(&right.last_maintenance_at)
                .then(left.tag.cmp(&right.tag)),
            _ => left.tag_normalized.cmp(&right.tag_normalized),
        };
        if descending {
            order.reverse()
        } else {
            order
        }
    });
    let total = valves.len();
    let start = page.saturating_sub(1).saturating_mul(page_size).min(total);
    let items = valves
        .into_iter()
        .skip(start)
        .take(page_size)
        .map(|valve| {
            let mut value = serde_json::to_value(valve).unwrap_or_else(|_| json!({}));
            value["health"] = json!(health_name(valve.last_maintenance_at.as_deref()));
            value["photos"] = json!(state
                .valve_photos
                .iter()
                .filter(|photo| photo.valve_id == valve.id)
                .collect::<Vec<_>>());
            value
        })
        .collect::<Vec<_>>();
    let zones = state
        .valves
        .iter()
        .map(|valve| valve.zone.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let valve_types = state
        .valves
        .iter()
        .filter_map(|valve| valve.valve_type.clone())
        .collect::<std::collections::BTreeSet<_>>();
    json!({
        "items": items,
        "schema_version": state.schema_version,
        "total": total,
        "page": page,
        "page_size": page_size,
        "facets": {"zones": zones, "valve_types": valve_types}
    })
}

fn overview_payload(state: &ApplicationState) -> Value {
    let mut ok = 0;
    let mut warning = 0;
    let mut critical = 0;
    for valve in &state.valves {
        match health_name(valve.last_maintenance_at.as_deref()) {
            "ok" => ok += 1,
            "warning" => warning += 1,
            _ => critical += 1,
        }
    }
    let orders_open = state
        .service_orders
        .iter()
        .filter(|order| !matches!(order.status, proexel_domain::ServiceOrderStatus::Completed))
        .count();
    let orders_in_progress = state
        .service_orders
        .iter()
        .filter(|order| matches!(order.status, proexel_domain::ServiceOrderStatus::InProgress))
        .count();
    let low_stock = state
        .stock_items
        .iter()
        .filter(|item| item.quantity <= item.minimum_quantity)
        .count();
    json!({
        "schema_version": state.schema_version,
        "valves": {"total": state.valves.len(), "ok": ok, "warning": warning, "critical": critical},
        "orders": {"open": orders_open, "in_progress": orders_in_progress, "completed": state.service_orders.len().saturating_sub(orders_open)},
        "stock": {"low": low_stock, "total": state.stock_items.len()},
        "recent_maintenance": state.maintenance_records.iter().rev().take(5).collect::<Vec<_>>(),
        "upcoming_orders": state.service_orders.iter().filter(|order| order.scheduled_for.is_some() && !matches!(order.status, proexel_domain::ServiceOrderStatus::Completed)).take(5).collect::<Vec<_>>()
    })
}

fn reports_payload(state: &ApplicationState) -> Value {
    let overview = overview_payload(state);
    let mut by_zone = BTreeMap::<String, (usize, usize, usize)>::new();
    let critical_valves = state
        .valves
        .iter()
        .filter_map(|valve| {
            let health = health_name(valve.last_maintenance_at.as_deref());
            let row = by_zone.entry(valve.zone.clone()).or_default();
            row.0 += 1;
            match health {
                "critical" => row.1 += 1,
                "warning" => row.2 += 1,
                _ => {}
            }
            (health == "critical").then(|| {
                json!({
                    "id": valve.id,
                    "tag": valve.tag,
                    "zone": valve.zone,
                    "last_maintenance_at": valve.last_maintenance_at,
                    "health": health,
                })
            })
        })
        .collect::<Vec<_>>();
    let zones = by_zone
        .into_iter()
        .map(|(zone, (total, critical, warning))| {
            json!({"zone": zone, "total": total, "critical": critical, "warning": warning})
        })
        .collect::<Vec<_>>();
    let recent_maintenance = state.maintenance_records.iter().rev().collect::<Vec<_>>();
    json!({
        "schema_version": state.schema_version,
        "generated_at_ms": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or_default(),
        "overview": overview,
        "by_zone": zones,
        "critical_valves": critical_valves,
        "recent_maintenance": recent_maintenance,
    })
}

fn health_name(last_maintenance: Option<&str>) -> &'static str {
    let days = last_maintenance.and_then(days_since_date);
    match maintenance_health(days) {
        MaintenanceHealth::Ok => "ok",
        MaintenanceHealth::Warning => "warning",
        MaintenanceHealth::Critical => "critical",
    }
}

fn days_since_date(value: &str) -> Option<u32> {
    let date = value.get(..10)?;
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    let then = days_from_civil(year, month, day)?;
    let now = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs()
        / 86_400) as i64;
    Some(now.saturating_sub(then).max(0) as u32)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let shifted_month = month as i32 + if month > 2 { -3 } else { 9 };
    let doy = (153 * shifted_month + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some((era * 146_097 + doe - 719_468) as i64)
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
        .join("target/runtime/storage/proexel-state-v1.json")
}

fn main() {
    let store = match JsonFileStore::new(state_path()) {
        Ok(store) => store,
        Err(error) => {
            eprintln!("proexel storage failed: {error}");
            std::process::exit(1);
        }
    };
    if let Err(error) = run_application(&ProexelApplication { store }) {
        eprintln!("proexel service failed: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_date_conversion_matches_unix_epoch() {
        assert_eq!(days_from_civil(1970, 1, 1), Some(0));
        assert_eq!(days_from_civil(2026, 8, 13), Some(20_678));
    }

    #[test]
    fn report_dataset_centralizes_zone_and_critical_counts() {
        let mut state = ApplicationState::default();
        state.valves.push(proexel_domain::Valve {
            id: "v1".to_string(),
            tag: "FV 1".to_string(),
            tag_normalized: "FV 1".to_string(),
            zone: "Linha 1".to_string(),
            manufacturer: None,
            serial: None,
            kit_reference: None,
            seat: None,
            dn: None,
            valve_type: None,
            actuator: None,
            manufactured_at: None,
            last_kit_changed_at: None,
            last_maintenance_at: None,
            created_at_ms: 0,
            updated_at_ms: 0,
        });

        let report = reports_payload(&state);

        assert_eq!(report["overview"]["valves"]["critical"], 1);
        assert_eq!(report["by_zone"][0]["zone"], "Linha 1");
        assert_eq!(report["by_zone"][0]["critical"], 1);
        assert_eq!(report["critical_valves"][0]["tag"], "FV 1");
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
            query_permission(commands::LIST_VALVES).unwrap()
        ));
        assert!(!can(
            Role::Compras,
            query_permission(commands::LIST_VALVES).unwrap()
        ));
        assert!(can(
            Role::Chefe,
            query_permission(commands::GET_REPORTS).unwrap()
        ));
    }

    #[test]
    fn production_volume_queries_are_paginated_and_bounded() {
        let started = std::time::Instant::now();
        let mut state = ApplicationState::default();
        for index in 0..2_000 {
            state.valves.push(proexel_domain::Valve {
                id: format!("v-{index}"),
                tag: format!("FV {index:05}"),
                tag_normalized: format!("FV {index:05}"),
                zone: format!("Zone {}", index % 20),
                manufacturer: None,
                serial: None,
                kit_reference: None,
                seat: None,
                dn: None,
                valve_type: Some("butterfly".to_string()),
                actuator: None,
                manufactured_at: None,
                last_kit_changed_at: None,
                last_maintenance_at: None,
                created_at_ms: index,
                updated_at_ms: index,
            });
            state.audit_events.push(proexel_domain::AuditEvent {
                id: format!("a-{index}"),
                actor: "Load test".to_string(),
                role: "admin".to_string(),
                operation: commands::CREATE_VALVE.to_string(),
                aggregate: "valve".to_string(),
                aggregate_id: format!("v-{index}"),
                description: Some("Valve created".to_string()),
                trace_id: None,
                before_json: None,
                after_json: None,
                result: "success".to_string(),
                created_at_ms: index,
            });
        }
        let valves = valves_payload(&state, &json!({"page": 20, "page_size": 50}));
        let audit = audit_payload(&state, &json!({"page": 10, "page_size": 100}));
        assert_eq!(valves["total"], 2_000);
        assert_eq!(valves["items"].as_array().unwrap().len(), 50);
        assert_eq!(audit["total"], 2_000);
        assert_eq!(audit["items"].as_array().unwrap().len(), 100);
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
    }
}
