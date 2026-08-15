use std::collections::BTreeMap;
use std::fmt;

use proexel_application::{ApplicationState, SCHEMA_VERSION};
use proexel_domain::AuditEvent;

pub const LEGACY_SOURCE_NAME: &str = "PROEXEL-main";
pub(crate) const LEGACY_CATEGORY_ID: &str = "legacy-category-valve";

struct PreparedMigration {
    candidate: ApplicationState,
    report: MigrationReport,
}

#[derive(Debug)]
enum MigrationPreparationError {
    BatchIdRequired,
    EncodeBundle(serde_json::Error),
}

impl fmt::Display for MigrationPreparationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BatchIdRequired => formatter.write_str("batch_id_required"),
            Self::EncodeBundle(error) => write!(formatter, "legacy_bundle_encode_failed: {error}"),
        }
    }
}

impl std::error::Error for MigrationPreparationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BatchIdRequired => None,
            Self::EncodeBundle(error) => Some(error),
        }
    }
}

pub fn migrate_bundle(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    batch_id: &str,
    now: u64,
    dry_run: bool,
) -> Result<MigrationReport, String> {
    let prepared = prepare_migration(bundle, state, batch_id, now, dry_run)
        .map_err(|error| error.to_string())?;
    if !dry_run {
        *state = prepared.candidate;
    }
    Ok(prepared.report)
}

fn prepare_migration(
    bundle: &LegacyBundle,
    state: &ApplicationState,
    batch_id: &str,
    now: u64,
    dry_run: bool,
) -> Result<PreparedMigration, MigrationPreparationError> {
    if batch_id.trim().is_empty() {
        return Err(MigrationPreparationError::BatchIdRequired);
    }
    let encoded = serde_json::to_vec(bundle).map_err(MigrationPreparationError::EncodeBundle)?;
    let mut candidate = state.clone();
    let mut report = MigrationReport {
        source: LEGACY_SOURCE_NAME.to_string(),
        batch_id: batch_id.to_string(),
        checksum: format!("fnv1a64:{:016x}", hash_bytes(&encoded)),
        dry_run,
        source_counts: source_counts(bundle),
        imported_counts: BTreeMap::new(),
        warnings: Vec::new(),
    };

    ensure_legacy_category(&mut candidate, now, &mut report);
    import_machines(bundle, &mut candidate, now, &mut report);
    import_machine_items(bundle, &mut candidate, now, &mut report);
    import_stock(bundle, &mut candidate, now, &mut report);
    import_inspections(bundle, &mut candidate, now, &mut report);
    import_orders(bundle, &mut candidate, now, &mut report);
    import_restock(bundle, &mut candidate, now, &mut report);
    import_suppliers(bundle, &mut candidate, now, &mut report);
    import_photos(bundle, &mut candidate, now, &mut report);
    refresh_machine_statuses(&mut candidate, now);
    candidate.schema_version = SCHEMA_VERSION;

    let imported = report.imported_counts.values().sum::<usize>();
    if imported > 0 {
        let imported_counts_json = match serde_json::to_string(&report.imported_counts) {
            Ok(encoded) => Some(encoded),
            Err(error) => {
                report
                    .warnings
                    .push(format!("migration audit counts encode failed: {error}"));
                None
            }
        };
        candidate.audit_events.push(AuditEvent {
            id: format!("audit-migration-{}", hash(batch_id)),
            actor: "migration-tool".to_string(),
            role: "system".to_string(),
            operation: "proexel.migration.import".to_string(),
            aggregate: "migration_batch".to_string(),
            aggregate_id: batch_id.to_string(),
            description: Some(format!(
                "Imported {imported} records into schema {SCHEMA_VERSION}"
            )),
            trace_id: Some(batch_id.to_string()),
            before_json: None,
            after_json: imported_counts_json,
            result: "success".to_string(),
            created_at_ms: now,
        });
    }
    Ok(PreparedMigration { candidate, report })
}

pub use legacy::{
    normalize_legacy_tag, LegacyBundle, LegacyMaintenance, LegacyOrder, LegacyPhoto, LegacyRestock,
    LegacyStock, LegacySupplier, LegacyValve,
};
pub use report::MigrationReport;

use asset_import::{ensure_legacy_category, import_machine_items, import_machines};
use inventory_import::{import_restock, import_stock, import_suppliers};
use maintenance_import::{import_inspections, import_orders};
use migration_support::{hash, hash_bytes, refresh_machine_statuses};
use photo_import::import_photos;
use report::source_counts;
mod asset_import;
mod inventory_import;
mod legacy;
mod maintenance_import;
mod migration_support;
mod photo_import;
mod report;
#[cfg(test)]
mod tests;
