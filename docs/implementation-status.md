# PROEXEL rebuild implementation status

This document records the outcome of each prompt-pack stage. It distinguishes a
completed engineering baseline from full legacy parity.

## 01 - Discovery and behavior map

**Status:** complete.

Created `legacy-behavior-map.md`, `legacy-data-map.md`, `rbac-matrix.md` and
`appcore-integration-surface.md` from the legacy source and AppCore contracts.
The executable parity gate is now maintained in
`functional-parity-checklist.md`.

**Decision:** preserve product behavior, replace Vite/Supabase/localStorage
architecture, normalize IDs/fields/statuses and discard the neon template.

## 02 - Target architecture

**Status:** baseline complete.

Created the independent `proexel/` workspace with domain, application,
infrastructure, migration, service and web boundaries. Added the AppCore
consumer ADR, application/deployment manifests and an architecture boundary
test.

**Decision:** AppCore owns generic lifecycle/transport/security primitives;
PROEXEL owns all business state, RBAC and conflict semantics.

## 03 - Bootstrap

**Status:** complete for local standalone development.

The AppCore-hosted Rust service starts with durable storage and health, while
the Next.js shell performs real queries and displays designed empty states. The
admin dashboard template was reduced to PROEXEL routes and navigation.

**Pending:** production service-manager/reverse-proxy artifacts are
environment-specific and not checked in.

## 04 - Domain and storage

**Status:** implemented baseline.

Implemented typed entities, schema-v1 state, normalization, maintenance health,
order transitions, non-negative stock, stock movements, idempotent command
receipts and atomic file replacement. Unit/integration tests cover the critical
policies and transaction rollback.

**Decision:** use a reliable single-writer JSON adapter for this baseline;
document that it is not a concurrent database or distributed sync solution.

## 05 - Auth, RBAC and audit

**Status:** implemented baseline.

Implemented server-side scrypt verification, signed expiring sessions, route
protection, four roles, permission checks in Next and the Rust application, and
semantic audit events on successful writes.

**Pending:** managed identity provisioning, shared rate limiting for multiple
web instances and a final policy for persisted denied-command audits.

## 06 - Admin dashboard UI

**Status:** primary routes complete; advanced workflows partial.

Implemented overview, valves, maintenance, orders, stock, purchasing,
suppliers, audit, reports, administration and settings with real queries and
writes, responsive layouts, localized states and permission-aware navigation.

**Pending:** valve detail/edit, richer filters/pagination, guided maintenance,
calendar scheduling, purchase cart and committed E2E coverage.

## 07 - Valves and maintenance

**Status:** core write path complete; media/signature UX partial.

Valve creation ensures stock idempotently. Maintenance updates the valve,
consumes one kit when available, records a stock movement, preserves physical
maintenance when consumption is pending and emits audit state atomically.

**Pending:** photo upload/blob adapter, graphical signature capture, per-valve
timeline and the complete guided review flow.

## 08 - Orders, stock, purchasing and suppliers

**Status:** primary flows complete.

Implemented order create/status transitions, stock upsert/adjustments,
restock create/review, supplier create/update, movement history and RBAC-backed
web routes.

**Pending:** selected delete/edit UI flows, schedule calendar, canonical order
priority enum and optional legacy purchase-cart/order-output workflow.

## 09 - Reports, i18n and offline

**Status:** reports dataset and i18n complete; offline/sync and PDF pending.

Implemented canonical report queries and visible reports. Added typed server and
client localization for Portuguese, English, Spanish and French, including
dates, accessibility and localized service errors. Browser QA covered login,
all protected routes and desktop/mobile layouts.

**Decision:** the standalone service is local-first durable operation, but no
remote sync is claimed. No service worker/localStorage fallback was added.

## 10 - Migration and compatibility

**Status:** tooling complete; production cutover not executed.

Implemented a separate deterministic importer with dry-run, checksum, batch
idempotency, aliases, warnings, audit event, fixture and JSON/Markdown reports.

**Pending:** validate against the real cutover export and migrate photo files
after selecting the target attachment provider.

## 11 - Tests, hardening and release

**Status:** baseline gates pass; production sign-off blocked by known product
work.

Latest verified gates:

- `npm run check`;
- `npx tsc --noEmit`;
- `npm run build`;
- `npm audit --audit-level=high`;
- `cargo test --workspace` (19 tests);
- `cargo audit`;
- browser smoke checks for four locales and all protected routes.

Final operational documentation now covers execution, architecture,
configuration, RBAC, backup/restore, migration, deployment, troubleshooting and
release status. Remaining blockers are listed in
`functional-parity-checklist.md`; none should be treated as delivered through
documentation alone.
