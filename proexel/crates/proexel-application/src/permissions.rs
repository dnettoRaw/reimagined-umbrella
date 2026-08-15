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
        "item_category.read" | "machine.read" => {
            matches!(role, Role::Admin | Role::Chefe | Role::Tecnico)
        }
        "item_category.manage" => matches!(role, Role::Admin),
        "machine.create" | "machine.update" | "machine_item.manage" => {
            matches!(role, Role::Admin | Role::Chefe)
        }
        "photo.manage_reference" => matches!(role, Role::Admin | Role::Chefe),
        "inspection.execute" => matches!(role, Role::Admin | Role::Chefe | Role::Tecnico),
        "inspection.read" => matches!(role, Role::Admin | Role::Chefe | Role::Tecnico),
        "order.read" => matches!(role, Role::Admin | Role::Chefe | Role::Tecnico),
        "operator.read" => matches!(role, Role::Admin | Role::Chefe),
        "order.create" | "order.manage" | "order.delete" => {
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
        "admin.manage" | "admin.users.manage" => matches!(role, Role::Admin),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{can, Role};

    #[test]
    fn technician_can_execute_inspection_but_cannot_adjust_stock() {
        assert!(can(Role::Tecnico, "inspection.execute"));
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
