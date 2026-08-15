use appcore_bin::application::{ApiRequest, ApiResponse, QueryEndpoint, QueryName, RuntimeResult};
use proexel_application::{can, commands, Role};
use proexel_infrastructure::JsonFileStore;
use serde_json::{json, Value};

pub(crate) struct ProexelQuery {
    pub(crate) name: QueryName,
    pub(crate) store: JsonFileStore,
}

#[derive(Debug)]
pub(crate) enum QueryPayloadError {
    InvalidJson(serde_json::Error),
}

pub(crate) struct ParsedQuery {
    role: Option<Role>,
    filters: Value,
}

pub(crate) fn parse_query_payload(payload: &[u8]) -> Result<ParsedQuery, QueryPayloadError> {
    let envelope: Value =
        serde_json::from_slice(payload).map_err(QueryPayloadError::InvalidJson)?;
    let role = envelope
        .pointer("/actor/role")
        .cloned()
        .and_then(|value| serde_json::from_value::<Role>(value).ok());
    let filters = envelope.get("data").cloned().unwrap_or_else(|| json!({}));
    Ok(ParsedQuery { role, filters })
}

impl QueryEndpoint for ProexelQuery {
    fn query_name(&self) -> &QueryName {
        &self.name
    }

    fn handle_query(&self, request: ApiRequest) -> RuntimeResult<ApiResponse> {
        let state = match self.store.read() {
            Ok(state) => state,
            Err(error) => {
                return crate::query_response::json_response(503, json!({"error": error}))
            }
        };
        let parsed = match parse_query_payload(&request.payload) {
            Ok(parsed) => parsed,
            Err(QueryPayloadError::InvalidJson(error)) => {
                eprintln!("proexel query payload rejected reason={error}");
                return crate::query_response::json_response(
                    400,
                    json!({"error": "invalid_query_payload"}),
                );
            }
        };
        let identity_query = self.name.as_str() == commands::RESOLVE_IDENTITY;
        let permission = query_permission(self.name.as_str());
        if !identity_query {
            let Some(role) = parsed.role else {
                return crate::query_response::json_response(403, json!({"error": "forbidden"}));
            };
            if permission.is_some_and(|permission| !can(role, permission)) {
                return crate::query_response::json_response(403, json!({"error": "forbidden"}));
            }
        }
        let filters = parsed.filters;
        let payload = match self.name.as_str() {
            commands::GET_OVERVIEW => crate::summary_queries::overview_payload(&state),
            commands::LIST_ITEM_CATEGORIES => {
                crate::asset_queries::categories_payload(&state, &filters)
            }
            commands::LIST_MACHINES => crate::asset_queries::machines_payload(&state, &filters),
            commands::LIST_SERVICE_ORDERS => {
                crate::operation_queries::orders_payload(&state, &filters)
            }
            commands::LIST_INSPECTIONS => {
                crate::operation_queries::inspections_payload(&state, &filters)
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
            commands::LIST_AUDIT => crate::operation_queries::audit_payload(&state, &filters),
            commands::GET_REPORTS => crate::summary_queries::reports_payload(&state),
            commands::LIST_USERS => crate::summary_queries::users_payload(&state),
            commands::LIST_OPERATORS => crate::summary_queries::operators_payload(&state),
            commands::RESOLVE_IDENTITY => {
                crate::summary_queries::identity_payload(&state, &filters)
            }
            _ => json!({"error": "unknown_query"}),
        };
        crate::query_response::json_response(200, payload)
    }
}

pub(crate) fn query_permission(name: &str) -> Option<&'static str> {
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
