# PROEXEL RBAC matrix - CONCLUÍDO 100%

Verified on 2026-08-15. The Rust application layer is authoritative; Next.js
duplicates the matrix for navigation and early rejection only.

| Permission | Admin | Chefe | Compras | Tecnico |
|---|---:|---:|---:|---:|
| `item_category.read` | Allow | Allow | Deny | Allow |
| `item_category.manage` | Allow | Deny | Deny | Deny |
| `machine.read` | Allow | Allow | Deny | Allow |
| `machine.create` / `machine.update` | Allow | Allow | Deny | Deny |
| `machine_item.manage` | Allow | Allow | Deny | Deny |
| `photo.manage_reference` | Allow | Allow | Deny | Deny |
| `inspection.read` / `inspection.execute` | Allow | Allow | Deny | Allow |
| `order.read` | Allow | Allow | Deny | Allow |
| `order.create` / `order.manage` / `order.delete` | Allow | Allow | Deny | Deny |
| `operator.read` | Allow | Allow | Deny | Deny |
| `restock.create_suggestion` | Deny | Deny | Deny | Allow |
| `restock.read` | Allow | Allow | Allow | Deny |
| `restock.approve_reject` / `restock.delete` | Allow | Allow | Deny | Deny |
| `stock.read` / add / adjust / delete | Allow | Allow | Allow | Deny |
| `supplier.read` / manage | Allow | Deny | Deny | Deny |
| `report.read` | Allow | Allow | Deny | Deny |
| `audit.read` | Allow | Allow | Deny | Deny |
| `admin.users.manage` | Allow | Deny | Deny | Deny |

## Technical-level authorization

RBAC is necessary but not sufficient for execution. Active users have
`maximum_repair_level` in 1..5, and each task freezes a component complexity in
1..5. Assignment, order/task start and inspection completion reject an operator
whose level is lower than the task complexity. A technician can act only as the
session identity; admin and chefe may act on behalf of an eligible operator.

## User administration

Admin can create accounts, change role and maximum level, activate/deactivate,
reset password/PIN and remove a PIN. The final active admin cannot be disabled.
Credential updates increment `auth_version`, invalidating existing sessions.
All operations produce redacted audit events.

## Enforcement locations

- Rust matrix: `crates/proexel-application/src/permissions.rs`
- Domain/application level checks: `asset_commands.rs`
- Web mirror: `apps/web/src/lib/proexel/permissions.ts`
- Route/session checks: `auth-server.ts` and dashboard server components
- Capability mapping: `apps/web/src/lib/proexel/service.ts`
