# Release notes - CONCLUÍDO 100%

## 0.2.0 - machine and guided-maintenance architecture

- Replaced the valve-centered runtime model with Machine, MachineItem,
  ItemCategory, structured MaintenanceGuide and ItemInspection.
- Added dynamic typed component fields and reusable category procedures editable
  without schema or code changes.
- Modeled a stable functional component position separately from its installed
  physical unit, including audited replacement history and equivalent parts.
- Added domain complexity and operator maximum repair levels from 1 through 5,
  enforced during assignment, start and completion.
- Added machine/some/all-component service orders with immutable machine, item,
  category, guide, complexity and guide-photo snapshots.
- Added a dedicated guided execution UI with structured step results, findings,
  before/during/after/defect/evidence photos and deterministic status updates.
- Added machine, component, guide, inspection and replacement photo ownership.
- Added complete component history, including inspections, findings, photos and
  old/new serial numbers after physical replacement.
- Added administrative category, machine, order, execution and operator flows.
- Added user creation/editing, roles, maximum level, password/PIN management,
  activation and redacted audit history.
- Updated overview, reports, PDF, notifications and audit for the new domain.
- Added typed Portuguese, English, Spanish and French translations for all
  product-owned visible text and server errors.
- Added deterministic legacy import and automatic canonical schema-v1 to v2
  migration with backup.
- Removed legacy runtime routes, commands, queries, DTOs, pages, translation keys
  and the old maintenance domain module.
- Updated the local launcher to issue tokens for every schema-v2 capability.
- Replaced the browser workflow with a complete machine/category/order/inspection
  scenario against an isolated real AppCore stack.
- Split oversized Rust modules into responsibility-specific domain, command,
  query, migration and transport modules while preserving public APIs.
- Added a structural gate for the 500-line production-file limit and declarative
  `mod.rs` files.
- Added pre-read limits for canonical state, migration input and web attachments,
  with explicit errors instead of silent serialization or payload fallbacks.

## Quality evidence

- Rust workspace tests: 33 passed.
- Rust structural gate: passed.
- Clippy all targets with warnings denied: passed.
- Biome, TypeScript and Next.js production build: passed.
- Playwright isolated integration workflow: passed.
- npm high-severity audit: zero vulnerabilities.

## Deployment note

Production cutover still requires the operator to supply and reconcile the real
legacy export, copy referenced binaries and execute the documented backup/restore
drill. These are environment-specific release operations; the migration and
runbook support is implemented.
