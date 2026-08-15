use std::collections::BTreeMap;

use proexel_application::ApplicationState;
use proexel_domain::{OperationalStatus, ServiceOrderStatus};
use serde_json::Value;

pub(crate) fn status_counts(
    statuses: impl Iterator<Item = OperationalStatus>,
) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for status in statuses {
        *counts.entry(operational_status_name(status)).or_default() += 1;
    }
    counts
}

pub(crate) fn order_status_counts(state: &ApplicationState) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for status in ["pending", "in_progress", "completed", "cancelled"] {
        counts.insert(status, 0);
    }
    for order in &state.service_orders {
        *counts.entry(order_status_name(order.status)).or_default() += 1;
    }
    counts
}

pub(crate) fn operational_status_name(status: OperationalStatus) -> &'static str {
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

pub(crate) fn order_status_name(status: ServiceOrderStatus) -> &'static str {
    match status {
        ServiceOrderStatus::Pending => "pending",
        ServiceOrderStatus::InProgress => "in_progress",
        ServiceOrderStatus::Completed => "completed",
        ServiceOrderStatus::Cancelled => "cancelled",
    }
}

pub(crate) fn text_filter(filters: &Value, key: &str) -> String {
    filters
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

pub(crate) fn normalized_filter(filters: &Value, key: &str) -> String {
    text_filter(filters, key).to_uppercase()
}

pub(crate) fn page_filter(filters: &Value, key: &str, default: usize, maximum: usize) -> usize {
    filters
        .get(key)
        .and_then(Value::as_u64)
        .unwrap_or(default as u64)
        .max(1)
        .min(maximum as u64) as usize
}

pub(crate) fn now_ms() -> u64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(error) => {
            eprintln!("proexel system clock is before unix epoch reason={error}");
            0
        }
    }
}
