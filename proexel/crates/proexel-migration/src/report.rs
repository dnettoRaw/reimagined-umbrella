use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::LegacyBundle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    pub source: String,
    pub batch_id: String,
    pub checksum: String,
    pub dry_run: bool,
    pub source_counts: BTreeMap<String, usize>,
    pub imported_counts: BTreeMap<String, usize>,
    pub warnings: Vec<String>,
}

pub(crate) fn source_counts(bundle: &LegacyBundle) -> BTreeMap<String, usize> {
    BTreeMap::from([
        ("valves".into(), bundle.valves.len()),
        (
            "maintenance_records".into(),
            bundle.maintenance_records.len(),
        ),
        ("orders".into(), bundle.orders.len()),
        ("restock_requests".into(), bundle.restock_requests.len()),
        ("stock".into(), bundle.stock.len()),
        ("suppliers".into(), bundle.suppliers.len()),
        ("valve_photos".into(), bundle.valve_photos.len()),
    ])
}

pub(crate) fn inc(report: &mut MigrationReport, name: &str) {
    *report.imported_counts.entry(name.to_string()).or_default() += 1;
}
