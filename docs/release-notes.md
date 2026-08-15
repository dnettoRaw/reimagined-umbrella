# Release notes

## Unreleased

- Added canonical user management for creation, four-role assignment,
  activation/deactivation, scrypt password/PIN reset and immediate session
  invalidation.
- Added redacted user audit history and a server-only identity resolution
  capability; credential hashes never appear in list or audit responses.
- Added stable-ID valve photo upload/removal and protected graphical maintenance signatures.
- Added valve detail/edit/timeline, production filters, facets, sorting and pagination.
- Added audited delete policies, canonical order priorities, schedule criticality,
  supplier validation and grouped purchase output.
- Added paginated localized PDF export and deduplicated operational notifications.
- Added backend audit filters/pagination/detail and runtime-derived standalone/degraded UX.
- Added committed Playwright E2E, concurrent storage and production-volume tests.
- Added complete UI localization for Portuguese, English, Spanish, and French,
  including authentication, navigation, forms, dialogs, empty/error states, and
  server-facing error presentation.
- Added consolidated architecture, configuration, deployment, backup/restore,
  troubleshooting, implementation-status, and functional-parity documents.
- Reconciled RBAC, migration, operations, AppCore boundary, and README guidance
  with the implemented baseline.

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

Known release blockers are real legacy-export reconciliation, source attachment
transfer, target failure drills,
and distributed sync/rate limiting only if that deployment topology is selected.
