# Release notes

## 0.1.0 - reconstruction baseline

- Added AppCore-hosted PROEXEL commands and queries with scoped capabilities.
- Added durable schema-v1 local state, atomic commits, idempotency, RBAC, and
  semantic audit events.
- Added valve, maintenance, service-order, stock, restock, supplier, audit,
  overview, report, administration, and runtime-status interfaces.
- Added deterministic legacy migration with dry-run, checksum, warnings, and
  JSON/Markdown reports.
- Added a canonical reports query for overview, zone, critical-valve, and recent
  maintenance datasets.
- Aligned visible web commands and server proxy authorization with the backend
  role matrix.
- Replaced template auth demos with scrypt password verification, signed
  `HttpOnly` sessions, rate limiting, protected routes, and session-backed audit
  identity.
- Removed dashboard template demo routes and legacy runtime dependencies.

Known release blockers for production remain: managed identity provisioning,
PT/EN/ES localization, validated photo upload/storage, configured sync/outbox,
and paginated PDF report export.
