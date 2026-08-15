use std::collections::BTreeSet;

use proexel_application::ApplicationState;
use proexel_domain::{normalize_reference, RestockRequest, StockItem, Supplier};

use crate::{
    migration_support::{clean, hash, restock_status},
    report::inc,
    LegacyBundle, MigrationReport,
};

pub(crate) fn import_stock(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.stock {
        let reference = normalize_reference(&old.reference);
        if reference.is_empty() {
            report
                .warnings
                .push("stock skipped: empty reference".to_string());
            continue;
        }
        if state
            .stock_items
            .iter()
            .any(|item| item.reference_normalized == reference)
        {
            continue;
        }
        if old.manufacturer.is_none() && old.brand.is_some() && old.location.is_some() {
            report.warnings.push(format!(
                "stock {reference}: brand mapped to manufacturer; location preserved"
            ));
        }
        state.stock_items.push(StockItem {
            id: format!("legacy-stock-{}", hash(&reference)),
            reference: reference.clone(),
            reference_normalized: reference,
            quantity: old.quantity,
            minimum_quantity: old.min_quantity,
            manufacturer: clean(&old.manufacturer).or_else(|| clean(&old.brand)),
            location: clean(&old.location),
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "stock_items");
    }
    let references = state
        .machine_items
        .iter()
        .filter_map(|item| item.replacement_specification.part_number.clone())
        .collect::<BTreeSet<_>>();
    for reference in references {
        if state
            .stock_items
            .iter()
            .any(|item| item.reference_normalized == reference)
        {
            continue;
        }
        state.stock_items.push(StockItem {
            id: format!("legacy-stock-auto-{}", hash(&reference)),
            reference: reference.clone(),
            reference_normalized: reference,
            quantity: 0,
            minimum_quantity: 0,
            manufacturer: None,
            location: None,
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "stock_items");
    }
}

pub(crate) fn import_restock(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.restock_requests {
        let reference = normalize_reference(&old.reference);
        let id = format!(
            "legacy-restock-{}",
            hash(&format!("{}|{}", reference, old.reason))
        );
        if state
            .restock_requests
            .iter()
            .any(|request| request.id == id)
        {
            continue;
        }
        state.restock_requests.push(RestockRequest {
            id,
            reference,
            reason: old.reason.clone(),
            requested_by: old
                .requested_by
                .clone()
                .unwrap_or_else(|| "legacy".to_string()),
            status: restock_status(&old.status),
            reviewed_by: None,
            reviewed_at_ms: None,
            created_at_ms: now,
        });
        inc(report, "restock_requests");
    }
}

pub(crate) fn import_suppliers(
    bundle: &LegacyBundle,
    state: &mut ApplicationState,
    now: u64,
    report: &mut MigrationReport,
) {
    for old in &bundle.suppliers {
        if old.name.trim().is_empty() || old.contact.trim().is_empty() {
            report.warnings.push(format!(
                "supplier skipped: name/contact required ({})",
                old.name
            ));
            continue;
        }
        let id = format!("legacy-supplier-{}", hash(&old.name.trim().to_uppercase()));
        if state.suppliers.iter().any(|supplier| supplier.id == id) {
            continue;
        }
        state.suppliers.push(Supplier {
            id,
            name: old.name.trim().to_string(),
            contact: old.contact.trim().to_string(),
            email: clean(&old.email),
            website: clean(&old.website),
            notes: clean(&old.notes),
            created_by: old
                .created_by
                .clone()
                .unwrap_or_else(|| "legacy".to_string()),
            created_at_ms: now,
            updated_at_ms: now,
        });
        inc(report, "suppliers");
    }
}
