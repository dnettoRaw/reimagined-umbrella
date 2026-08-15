# PROEXEL implementation status - CONCLUÍDO 100%

Status verified against code and automated gates on 2026-08-15.

## Completed scope

| Area | Status | Evidence |
|---|---:|---|
| Discovery and boundaries | 100% | Legacy maps, AppCore boundary ADR and dependency test |
| Domain and storage | 100% | Machine/category/item/guide/inspection model, schema v2 and migration tests |
| Auth, RBAC and operators | 100% | Four roles, audited users, password/PIN, max level 1..5 |
| Admin UI | 100% | Machines, categories, guide editor, orders, execution, users and audit |
| Guided maintenance | 100% | Typed steps, reference/evidence photos, immutable snapshots and histories |
| Operations | 100% | Stock, purchasing, suppliers, reports, notifications and PDF |
| Migration | 100% | Deterministic legacy and in-place schema migration paths |
| Localization | 100% | Typed PT/EN/ES/FR catalogs for all visible product text |
| Release gates | 100% | Tests, clippy, Biome, TypeScript, build, E2E and npm audit |

## Canonical architecture

The runtime model is `Machine -> MachineItem[] -> ItemCategory -> MaintenanceGuide`.
`MachineItem` is a stable functional position and `InstalledComponent` is the
replaceable physical unit. `ServiceOrderTask` and `ItemInspection` preserve the
historical context needed for audit and reporting.

Legacy Valve DTOs are isolated to migration code. There are no legacy runtime
routes, commands, queries, domain modules, UI pages or translation keys.

## Supported topology

The completed topology is standalone local read/write through AppCore capability
hosting. Remote sync is intentionally not configured, so the UI does not present
pending/conflict states or claim remote persistence.

## Verified gates

- 29 Rust tests passed across domain, application, infrastructure, migration and service.
- Clippy passed for every workspace target with warnings denied.
- Biome and TypeScript passed for the web application.
- Next.js generated the production route tree without legacy routes.
- Playwright completed the machine-to-inspection workflow on the isolated stack.
- npm reported zero high-severity vulnerabilities.
