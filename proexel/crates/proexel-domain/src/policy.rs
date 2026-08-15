use crate::model::{ComplexityLevel, OperationalStatus, ServiceOrderStatus};

pub fn normalize_identifier(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase()
}

pub fn normalize_tag(value: &str) -> String {
    normalize_identifier(value)
}

pub fn normalize_reference(value: &str) -> String {
    normalize_identifier(value)
}

pub fn can_transition_order(from: ServiceOrderStatus, to: ServiceOrderStatus) -> bool {
    from == to
        || matches!(
            (from, to),
            (ServiceOrderStatus::Pending, ServiceOrderStatus::InProgress)
                | (ServiceOrderStatus::Pending, ServiceOrderStatus::Completed)
                | (ServiceOrderStatus::InProgress, ServiceOrderStatus::Pending)
                | (
                    ServiceOrderStatus::InProgress,
                    ServiceOrderStatus::Completed
                )
                | (ServiceOrderStatus::Pending, ServiceOrderStatus::Cancelled)
                | (
                    ServiceOrderStatus::InProgress,
                    ServiceOrderStatus::Cancelled
                )
        )
}

pub fn can_execute_complexity(operator: ComplexityLevel, required: ComplexityLevel) -> bool {
    operator >= required
}

pub fn derive_machine_status<'a>(
    statuses: impl IntoIterator<Item = &'a OperationalStatus>,
) -> OperationalStatus {
    let mut derived = OperationalStatus::Unknown;
    for status in statuses {
        derived = match (derived, status) {
            (_, OperationalStatus::Critical) => OperationalStatus::Critical,
            (OperationalStatus::Critical, _) => OperationalStatus::Critical,
            (_, OperationalStatus::MaintenanceRequired) => OperationalStatus::MaintenanceRequired,
            (OperationalStatus::MaintenanceRequired, _) => OperationalStatus::MaintenanceRequired,
            (_, OperationalStatus::UnderMaintenance) => OperationalStatus::UnderMaintenance,
            (OperationalStatus::UnderMaintenance, _) => OperationalStatus::UnderMaintenance,
            (_, OperationalStatus::Attention) => OperationalStatus::Attention,
            (OperationalStatus::Attention, _) => OperationalStatus::Attention,
            (_, OperationalStatus::Ok) => OperationalStatus::Ok,
            (OperationalStatus::Ok, _) => OperationalStatus::Ok,
            (_, OperationalStatus::Disabled) => OperationalStatus::Disabled,
            (OperationalStatus::Disabled, _) => OperationalStatus::Disabled,
            _ => OperationalStatus::Unknown,
        };
    }
    derived
}

pub fn adjust_stock(quantity: u32, delta: i32) -> Result<u32, &'static str> {
    let adjusted = i64::from(quantity) + i64::from(delta);
    if adjusted < 0 {
        return Err("stock_cannot_be_negative");
    }
    u32::try_from(adjusted).map_err(|_| "stock_quantity_overflow")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_tags_and_references_once() {
        assert_eq!(normalize_tag("  fv   10.2 "), "FV 10.2");
        assert_eq!(normalize_reference(" kit  ab-2 "), "KIT AB-2");
    }

    #[test]
    fn completed_order_cannot_be_reopened() {
        assert!(!can_transition_order(
            ServiceOrderStatus::Completed,
            ServiceOrderStatus::InProgress
        ));
        assert!(can_transition_order(
            ServiceOrderStatus::Pending,
            ServiceOrderStatus::Completed
        ));
    }

    #[test]
    fn complexity_and_machine_status_are_domain_rules() {
        assert!(can_execute_complexity(
            ComplexityLevel::new(4).unwrap(),
            ComplexityLevel::new(3).unwrap()
        ));
        assert!(!can_execute_complexity(
            ComplexityLevel::new(2).unwrap(),
            ComplexityLevel::new(3).unwrap()
        ));
        assert_eq!(
            derive_machine_status([OperationalStatus::Ok, OperationalStatus::Critical].iter()),
            OperationalStatus::Critical
        );
    }

    #[test]
    fn stock_never_becomes_negative() {
        assert_eq!(adjust_stock(3, -2), Ok(1));
        assert_eq!(adjust_stock(0, -1), Err("stock_cannot_be_negative"));
    }
}
