use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceHealth {
    Ok,
    Warning,
    Critical,
}

pub fn maintenance_health(days_since_last_maintenance: Option<u32>) -> MaintenanceHealth {
    match days_since_last_maintenance {
        None => MaintenanceHealth::Critical,
        Some(days) if days > 180 => MaintenanceHealth::Critical,
        Some(days) if days > 150 => MaintenanceHealth::Warning,
        Some(_) => MaintenanceHealth::Ok,
    }
}

#[cfg(test)]
mod tests {
    use super::{maintenance_health, MaintenanceHealth};

    #[test]
    fn missing_maintenance_is_critical() {
        assert_eq!(maintenance_health(None), MaintenanceHealth::Critical);
    }

    #[test]
    fn more_than_180_days_is_critical() {
        assert_eq!(maintenance_health(Some(181)), MaintenanceHealth::Critical);
    }

    #[test]
    fn more_than_150_days_is_warning_until_180() {
        assert_eq!(maintenance_health(Some(151)), MaintenanceHealth::Warning);
        assert_eq!(maintenance_health(Some(180)), MaintenanceHealth::Warning);
    }

    #[test]
    fn up_to_150_days_is_ok() {
        assert_eq!(maintenance_health(Some(150)), MaintenanceHealth::Ok);
        assert_eq!(maintenance_health(Some(0)), MaintenanceHealth::Ok);
    }
}
