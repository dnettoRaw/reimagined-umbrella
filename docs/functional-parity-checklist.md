# PROEXEL functional completion checklist - CONCLUÍDO 100%

Verified 2026-08-15 for the supported standalone topology.

## Platform and security

- [x] PROEXEL domain remains outside generic AppCore crates.
- [x] Explicit command/query capabilities, atomic schema-v2 storage and idempotent writes.
- [x] Automatic schema-v1 backup and migration.
- [x] Scrypt password/PIN authentication and signed, versioned `HttpOnly` sessions.
- [x] Four-role RBAC enforced in Rust, server routes and UI.
- [x] User creation, role, maximum level, activation, password/PIN reset and redacted history.
- [x] Semantic audit events include actor, role, timestamp, entity, trace and before/after values.

## Machines, components and categories

- [x] Dynamic `ItemCategory` with typed custom fields, recommended parts and active state.
- [x] Friendly structured guide editor with nine step types and reordering.
- [x] Machine create/edit/detail, photos and deterministic status.
- [x] Component add/edit/reorder/remove with default or overridden complexity 1..5.
- [x] Functional position separated from replaceable installed physical unit.
- [x] Full replacement history with old/new identity, reason, actor and timestamp.
- [x] Replacement specifications, technical properties and equivalent parts editable in UI.
- [x] Active and removed component histories remain consultable.

## Guided execution

- [x] OS selects one, some or all current components and freezes the selection.
- [x] Machine, item, category, guide version/content, complexity and guide photos are snapshotted.
- [x] Independent tasks can be assigned only to eligible operators.
- [x] Operator level is checked in the backend when assigning, starting and completing.
- [x] Step-by-step execution persists structured results and findings.
- [x] Required, typed choice/number/text/photo and measurement-unit validations are enforced.
- [x] Inspection evidence is linked to the inspection and shown in component history.
- [x] Item and machine states update deterministically after inspection.
- [x] Orders cannot close with pending tasks.

## Operations and localization

- [x] Stock, non-negative adjustments, restock review, suppliers and purchase outputs remain available.
- [x] Overview, reports, PDF, notifications and paginated audit use the new model.
- [x] Portuguese, English, Spanish and French dictionaries have typed key parity.
- [x] Old pages, API routes, DTOs, commands, queries and translation keys were removed.
- [x] Legacy Valve names remain only in import DTOs/maps required to read old exports.

## Migration and quality gates

- [x] Deterministic legacy CLI with dry-run, checksum idempotency, warnings and reports.
- [x] Legacy records map to Machine, MachineItem, ItemInspection, snapshots and category Valve.
- [x] Rust workspace tests pass, including migration, storage concurrency and volume queries.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [x] Biome, TypeScript and Next.js production build pass.
- [x] Playwright passes against an isolated stack with real AppCore capabilities.
- [x] `npm audit --audit-level=high` reports zero vulnerabilities.

Production export reconciliation, binary transfer and target restore drills are
deployment-runbook activities, not unfinished application behavior.
