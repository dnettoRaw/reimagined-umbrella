# PROEXEL functional parity checklist

Status date: 2026-08-15. This is the migration/release gate derived from the
legacy discovery. A checked item is implemented in the rebuilt code, not merely
documented. `Partial` means the underlying contract exists but the complete
operator workflow does not.

## Platform and boundaries

- [x] New Rust/Next.js application is independent of the legacy Vite/Supabase
  runtime.
- [x] AppCore remains product-independent and is consumed through public
  application/runtime contracts.
- [x] Domain, application, infrastructure, migration, service and web
  responsibilities are separated.
- [x] Commands and queries use an explicit capability allowlist.
- [x] Writes require idempotency keys and retain replay receipts.
- [x] Canonical state has an explicit schema version and durable atomic file
  replacement for the standalone single-writer topology.
- [ ] Configure outbox/remote sync, conflict ownership and sync-state UX if a
  distributed deployment is required.

## Authentication, RBAC and audit

- [x] Server-side scrypt password verification.
- [x] Signed expiring `HttpOnly`, `SameSite=Strict` session cookie.
- [x] Login attempt throttling for the current single Next.js process.
- [x] Roles `admin`, `chefe`, `compras` and `tecnico`.
- [x] Backend/application permission checks for every implemented command and
  query.
- [x] Permission-aware navigation and actions in the UI.
- [x] Successful writes create semantic local audit events with actor, role,
  operation, aggregate, before/after where available and trace ID.
- [ ] Provision production identities through a managed process.
- [ ] Add shared/durable rate limiting before horizontally scaling Next.js.
- [ ] Decide and test whether denied/failed commands require persisted audit
  events in addition to runtime logs.

## Valves

- [x] Stable internal valve ID; TAG is not the primary key.
- [x] TAG normalization and uniqueness.
- [x] Create and update commands preserve the main technical fields.
- [x] Creating a valve with a kit reference idempotently ensures a stock item
  without overwriting existing stock.
- [x] Search by normalized TAG and zone.
- [x] Responsive list, empty state and create form.
- [ ] Add UI editing and a valve detail view with technical data and timeline.
- [ ] Add zone/status/type filters, sorting and pagination for production volume.
- [ ] Implement validated valve photo upload, immutable `valve_id` association,
  blob lifecycle and thumbnail serving. Migration currently preserves metadata
  only.

## Maintenance

- [x] Canonical policy: never maintained and more than 180 days are critical;
  151-180 days are warning; up to 150 days is OK.
- [x] Unit tests cover never, 150, 151, 180 and 181-day boundaries.
- [x] Register preventive/corrective maintenance with technician, service,
  notes, date, signature reference and kit-change flag.
- [x] Registration updates valve maintenance/kit dates.
- [x] Kit consumption decrements exactly one unit transactionally and is
  idempotent on retry.
- [x] Stock never becomes negative; zero stock preserves the physical
  maintenance and marks consumption pending.
- [x] Global maintenance history is query-backed.
- [ ] Implement the guided maintenance procedure and review step.
- [ ] Implement graphical mouse/touch signature capture, secure storage and
  permission-controlled viewing.
- [ ] Add per-valve timeline/detail workflow and searchable global filters.
- [ ] Add automated browser E2E for valve -> maintenance -> stock -> audit.

## Service orders and schedule

- [x] Canonical fields for zone, optional valve, description, priority, status,
  creator, technician and scheduled date.
- [x] Legacy status aliases normalize during migration.
- [x] State-transition policy prevents reopening completed orders.
- [x] Create and status-change commands, UI and RBAC.
- [ ] Implement audited order deletion if the legacy behavior remains required.
- [ ] Add calendar/planning view and zone criticality indicator.
- [ ] Replace free-form priority in the domain model with a canonical enum.

## Stock, restock and suppliers

- [x] Unique normalized stock references with separate manufacturer/location.
- [x] Non-negative stock invariant and required reason for manual adjustment.
- [x] Durable stock movements for receipt/consumption/correction/migration.
- [x] Restock create and approve/reject commands with reviewer and timestamp.
- [x] Supplier create/update commands and admin-only authorization.
- [x] Operational list/create/review/adjust UI flows.
- [ ] Add stock/restock deletion only after an explicit retention/audit decision.
- [ ] Add supplier edit/delete UI and stronger email/URL validation.
- [ ] Rebuild purchase cart, grouped kit demand and purchase-order/email output
  if confirmed as required operations.
- [ ] Add concurrency tests beyond process-local serialized transactions if the
  storage topology changes.

## Reports, notifications and localization

- [x] Backend report dataset for overview, zone criticality, critical valves and
  recent maintenance.
- [x] Query-backed reports UI.
- [x] Typed PT/EN/ES/FR catalog for authentication, navigation, primary pages,
  forms, errors and accessibility text.
- [x] Locale persistence and localized date/number formatting.
- [x] Desktop/mobile browser smoke validation for all four locales.
- [ ] Generate paginated PDF reports with locale, metadata and no silent record
  truncation.
- [ ] Rebuild actionable notifications for maintenance, stock, orders and
  runtime/sync status with duplicate suppression.

## Migration

- [x] Separate migration crate and CLI.
- [x] Dry-run, deterministic IDs, batch ID, checksum and idempotent rerun.
- [x] Legacy aliases for valves, maintenance, orders, restock, stock, suppliers
  and valve photo metadata.
- [x] JSON and Markdown reports with counts and warnings.
- [x] Passwords excluded from the import contract.
- [x] Example fixture and automated idempotency test.
- [ ] Validate a real production export and reconcile its counts/attachments
  before cutover.
- [ ] Add attachment file transfer once the target blob provider exists.

## Quality and release

- [x] Rust workspace tests, architecture boundary test and durable storage test.
- [x] Biome check, TypeScript check and production Next.js build.
- [x] Rust and npm dependency audits passed on the status date.
- [x] Responsive browser smoke test with no console errors on protected routes.
- [x] Architecture, configuration, RBAC, migration, backup/restore, deployment,
  troubleshooting and release documentation.
- [ ] Add committed E2E automation for representative roles and write flows.
- [ ] Add production-volume performance tests and audit/history pagination.
- [ ] Complete all product blockers above before production sign-off.
