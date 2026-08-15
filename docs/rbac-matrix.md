# PROEXEL RBAC matrix

Status: legacy evidence confirmed 2026-08-13; implementation reviewed
2026-08-15.

## Roles

Legacy roles are `admin`, `chefe`, `compras`, `tecnico`.

Evidence: `src/App.jsx:164`, `src/components/NavBar.jsx:40`, `src/components/Dashboard.jsx:23`.

## Page access

| Area | admin | chefe | compras | tecnico | Evidence |
|---|---:|---:|---:|---:|---|
| Dashboard | Allow | Allow | Deny | Deny | `src/App.jsx:164` |
| Valves | Allow | Allow | Deny | Allow | `src/App.jsx:164` |
| Agenda | Allow | Allow | Deny | Allow | `src/App.jsx:164` |
| Maintenance panel | Allow | Allow | Deny | Deny | `src/App.jsx:164` |
| Compras | Allow | Allow | Allow | Deny | `src/App.jsx:164` |
| Historico | Allow | Allow | Deny | Deny | `src/App.jsx:164` |
| Report PDF | Allow | Allow | Deny | Deny | `src/App.jsx:192`, `src/App.jsx:383` |

## Action access observed in UI

| Permission | admin | chefe | compras | tecnico | Evidence |
|---|---:|---:|---:|---:|---|
| `valve.read` | Allow | Allow | Deny | Allow | page access |
| `valve.create` | Allow | Allow | Deny | Deny | `src/components/Valves.jsx:46` |
| `valve.update_technical_fields` | Allow | Deny | Deny | Deny | `src/components/ValveDetail.jsx:7`, `src/components/ValveDetail.jsx:112` |
| `valve.update_photo` | Allow | Deny | Deny | Deny | `src/components/ValveDetail.jsx:7`, `src/components/ValveDetail.jsx:77` |
| `maintenance.guide` | Allow | Allow | Deny | Allow | any role with valve detail can start guide; no role check in `ValveDetail` |
| `maintenance.register` | Allow | Allow | Deny | Allow | any role with guide can finish; no role check in `MaintFinish` |
| `maintenance.history.read` | Allow | Allow | Deny | Allow per valve; admin/chefe global | `src/components/MaintHistory.jsx`; page access |
| `order.read` | Allow | Allow | Deny | Allow | page access |
| `order.create` | Allow | Allow | Deny | Deny | `src/components/Agenda.jsx:17`; `src/App.jsx:325` |
| `order.change_status` | Allow | Allow | Deny | Deny | `src/components/Agenda.jsx:20` |
| `order.delete` | Allow | Allow | Deny | Deny | `src/components/Agenda.jsx:17`, `src/components/Agenda.jsx:91` |
| `restock.create_suggestion` | Deny in UI path | Deny in UI path | Deny | Allow | `src/components/Agenda.jsx:108` |
| `restock.read` | Allow | Allow | Allow | Deny | `src/components/Agenda.jsx:18`; `src/components/Compras.jsx:380` |
| `restock.approve_reject` | Allow | Allow | Deny | Deny | `src/components/Agenda.jsx:151` |
| `restock.delete` | Allow | Allow | Deny | Deny | `src/components/Agenda.jsx:158` |
| `stock.read` | Allow | Allow | Allow | Deny | compras page access |
| `stock.add_or_increment` | Allow | Allow | Allow | Deny | `src/components/Compras.jsx:20`, `src/components/Compras.jsx:303` |
| `stock.adjust_quantity` | Allow | Allow | Allow | Deny | `src/components/Compras.jsx:20`, `src/components/Compras.jsx:365` |
| `stock.delete` | Allow | Allow | Allow | Deny | `src/components/Compras.jsx:20`, `src/components/Compras.jsx:367` |
| `supplier.read` | Allow | Deny in UI | Deny in UI | Deny | supplier panel gated by `isAdmin`, `src/components/Compras.jsx:426` |
| `supplier.create_update_delete` | Allow | Deny | Deny | Deny | `src/components/Compras.jsx:426` |
| `audit.write_suppliers` | Allow | Allow if called | Allow if called | Deny | calls are in `App.jsx:264`, but supplier UI is admin-only |
| `audit.read` | Not implemented as audit-log screen | Not implemented | Deny | Deny | `HistoricoGeral` shows maintenance history, not `audit_log` |

## SQL/RLS reality

The Supabase RLS production script is much broader than UI RBAC:

- `maintenance_records`: anon SELECT and INSERT, no UPDATE/DELETE.
- `orders`: anon SELECT, INSERT, UPDATE and DELETE.
- `restock_requests`: anon SELECT, INSERT and UPDATE; comments say no DELETE, but the UI hook still calls delete.
- `stock`: anon SELECT, INSERT, UPDATE and DELETE.
- `users`: direct anon/authenticated access revoked; login via SECURITY DEFINER `app_login`.

Evidence: `scripts/supabase-rls-production.sql:22`.

## Implemented application matrix

The following matrix is enforced by
`proexel/crates/proexel-application/src/permissions.rs`. Unknown permissions are
denied by default.

| Permission | admin | chefe | compras | tecnico |
|---|---:|---:|---:|---:|
| `valve.read` | Allow | Allow | Deny | Allow |
| `valve.create` | Allow | Allow | Deny | Deny |
| `valve.update_technical_fields` | Allow | Deny | Deny | Deny |
| `valve.update_photo` | Allow | Deny | Deny | Deny |
| `maintenance.read` | Allow | Allow | Deny | Allow |
| `maintenance.register` | Allow | Allow | Deny | Allow |
| `order.read` | Allow | Allow | Deny | Allow |
| `order.create`, `order.change_status`, `order.delete` | Allow | Allow | Deny | Deny |
| `restock.create_suggestion` | Deny | Deny | Deny | Allow |
| `restock.read` | Allow | Allow | Allow | Deny |
| `restock.approve_reject`, `restock.delete` | Allow | Allow | Deny | Deny |
| `stock.read`, `stock.add_or_increment`, `stock.adjust_quantity`, `stock.delete` | Allow | Allow | Allow | Deny |
| `supplier.read`, `supplier.create_update_delete` | Allow | Deny | Deny | Deny |
| `report.read`, `audit.read` | Allow | Allow | Deny | Deny |
| `admin.manage` | Allow | Deny | Deny | Deny |

This table describes policy decisions, not a claim that every listed legacy
action has a registered command. The current capability manifest does not
register order, restock, or stock deletion commands; those workflows remain
open in the functional parity checklist.

## Enforcement rules

- Application command/query handlers enforce permissions independently of the
  UI.
- Next.js route protection and command proxy checks mirror the backend matrix;
  hidden navigation is only a convenience layer.
- Supplier access remains admin-only, matching the observed legacy UI until a
  product decision changes it.
- `audit.read` is separate from `maintenance.read`; the rebuild includes a real
  audit view for admin and chefe.
- Auditing denied command/query attempts remains pending.
