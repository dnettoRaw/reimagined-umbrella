import type { Role } from "./types";

export function can(permission: string, role: Role): boolean {
  switch (permission) {
    case "item_category.read":
    case "machine.read":
    case "inspection.execute":
    case "inspection.read":
    case "order.read":
      return role === "admin" || role === "chefe" || role === "tecnico";
    case "operator.read":
      return role === "admin" || role === "chefe";
    case "machine.create":
    case "machine.update":
    case "machine_item.manage":
    case "photo.manage_reference":
    case "order.create":
    case "order.manage":
    case "order.delete":
    case "restock.approve_reject":
    case "restock.delete":
    case "report.read":
    case "audit.read":
      return role === "admin" || role === "chefe";
    case "item_category.manage":
    case "supplier.read":
    case "supplier.create_update_delete":
    case "admin.manage":
    case "admin.users.manage":
      return role === "admin";
    case "restock.create_suggestion":
      return role === "tecnico";
    case "restock.read":
    case "stock.read":
    case "stock.add_or_increment":
    case "stock.adjust_quantity":
    case "stock.delete":
      return role === "admin" || role === "chefe" || role === "compras";
    default:
      return false;
  }
}
