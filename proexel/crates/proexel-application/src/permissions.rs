use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Chefe,
    Compras,
    Tecnico,
}

pub fn can(role: Role, permission: &str) -> bool {
    match permission {
        "valve.read" => matches!(role, Role::Admin | Role::Chefe | Role::Tecnico),
        "valve.create" => matches!(role, Role::Admin | Role::Chefe),
        "valve.update_technical_fields" | "valve.update_photo" => matches!(role, Role::Admin),
        "maintenance.register" => matches!(role, Role::Admin | Role::Chefe | Role::Tecnico),
        "maintenance.read" => matches!(role, Role::Admin | Role::Chefe | Role::Tecnico),
        "order.read" => matches!(role, Role::Admin | Role::Chefe | Role::Tecnico),
        "order.create" | "order.change_status" | "order.delete" => {
            matches!(role, Role::Admin | Role::Chefe)
        }
        "restock.create_suggestion" => matches!(role, Role::Tecnico),
        "restock.read" => matches!(role, Role::Admin | Role::Chefe | Role::Compras),
        "restock.approve_reject" | "restock.delete" => matches!(role, Role::Admin | Role::Chefe),
        "stock.read" | "stock.add_or_increment" | "stock.adjust_quantity" | "stock.delete" => {
            matches!(role, Role::Admin | Role::Chefe | Role::Compras)
        }
        "supplier.read" | "supplier.create_update_delete" => matches!(role, Role::Admin),
        "report.read" => matches!(role, Role::Admin | Role::Chefe),
        "audit.read" => matches!(role, Role::Admin | Role::Chefe),
        "admin.manage" => matches!(role, Role::Admin),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{can, Role};

    #[test]
    fn technician_can_register_maintenance_but_cannot_adjust_stock() {
        assert!(can(Role::Tecnico, "maintenance.register"));
        assert!(!can(Role::Tecnico, "stock.adjust_quantity"));
    }

    #[test]
    fn compras_can_manage_stock_but_not_approve_restock_in_legacy_ui() {
        assert!(can(Role::Compras, "stock.adjust_quantity"));
        assert!(!can(Role::Compras, "restock.approve_reject"));
    }

    #[test]
    fn supplier_management_is_admin_only_until_redecided() {
        assert!(can(Role::Admin, "supplier.create_update_delete"));
        assert!(!can(Role::Chefe, "supplier.create_update_delete"));
    }
}
