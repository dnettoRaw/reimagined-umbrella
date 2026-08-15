import type { Role } from "./types";

export function can(permission: string, role: Role): boolean {
  switch (permission) {
    case "valve.read":
    case "maintenance.register":
    case "maintenance.read":
    case "order.read":
      return role === "admin" || role === "chefe" || role === "tecnico";
    case "valve.create":
    case "order.create":
    case "order.change_status":
    case "order.delete":
    case "restock.approve_reject":
    case "restock.delete":
    case "report.read":
    case "audit.read":
      return role === "admin" || role === "chefe";
    case "valve.update_technical_fields":
    case "valve.update_photo":
    case "supplier.read":
    case "supplier.create_update_delete":
    case "admin.manage":
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
