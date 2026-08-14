use std::collections::BTreeMap;
use std::path::PathBuf;

use appcore_bin::application::{
    run_application, ApiRequest, ApiResponse, ApiRouter, Application, CommandBus, CommandEnvelope,
    CommandHandler, CommandName, CommandRegistry, CommandResult, EventEnvelope, EventName,
    EventRegistry, QueryEndpoint, QueryName, RuntimeContext, RuntimeResult,
};
use proexel_application::{commands, ApplicationState};
use proexel_domain::{maintenance_health, MaintenanceHealth};
use proexel_infrastructure::JsonFileStore;
use serde_json::{json, Value};

const EVENTS: &[&str] = &[
    "valve.changed",
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
            Err(error) => return Ok(CommandResult::rejected(error)),
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
        let filters: Value = serde_json::from_slice(&request.payload).unwrap_or_else(|_| json!({}));
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
            commands::LIST_AUDIT => {
                json!({"items": state.audit_events.iter().rev().take(250).collect::<Vec<_>>(), "schema_version": state.schema_version})
            }
            commands::GET_REPORTS => reports_payload(&state),
            _ => json!({"error": "unknown_query"}),
        };
        json_response(200, payload)
    }
}

fn valves_payload(state: &ApplicationState, filters: &Value) -> Value {
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
    let items = state
        .valves
        .iter()
        .filter(|valve| {
            search.is_empty()
                || valve.tag_normalized.contains(&search)
                || valve.zone.to_uppercase().contains(&search)
        })
        .filter(|valve| zone.is_empty() || valve.zone == zone)
        .map(|valve| {
            let mut value = serde_json::to_value(valve).unwrap_or_else(|_| json!({}));
            value["health"] = json!(health_name(valve.last_maintenance_at.as_deref()));
            value
        })
        .collect::<Vec<_>>();
    json!({"items": items, "schema_version": state.schema_version})
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
    let recent_maintenance = state
        .maintenance_records
        .iter()
        .rev()
        .take(50)
        .collect::<Vec<_>>();
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
}
