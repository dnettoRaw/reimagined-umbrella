use crate::model::ServiceOrderStatus;

pub fn normalize_identifier(value: &str) -> String {
    value
        .trim()
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
        )
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
    fn stock_never_becomes_negative() {
        assert_eq!(adjust_stock(3, -2), Ok(1));
        assert_eq!(adjust_stock(0, -1), Err("stock_cannot_be_negative"));
    }
}
