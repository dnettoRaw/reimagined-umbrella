# PROEXEL functional parity checklist

Verified 2026-08-15. Checked items are implemented in the supported standalone
topology; unchecked items require code or external production evidence.

## Platform, auth and audit

- [x] Independent Rust/Next application consumes only public AppCore contracts.
- [x] Explicit command/query capabilities, schema-v1 atomic storage and idempotent writes.
- [x] Scrypt authentication, signed expiring secure sessions and process-local login throttling.
- [x] Roles `admin`, `chefe`, `compras`, `tecnico` enforced in UI, Next routes and Rust handlers.
- [x] Successful writes persist semantic, redacted audit events with trace IDs.
- [x] Denied/failed commands are redacted in operational logs and do not persist attacker-controlled audit rows.
- [ ] Provision production identities and secret rotation through the target managed platform.
- [ ] Add shared rate limiting only before horizontally scaling the Next.js tier.

## Valves and maintenance

- [x] Stable valve ID, normalized unique TAG and complete technical create/edit/detail flow.
- [x] Backend filters, sorting, facets and pagination by TAG, zone, health and type.
- [x] Stable-ID photo metadata with authenticated PNG/JPEG/WebP storage, magic-byte and 5 MB validation.
- [x] Canonical 150/180-day policy with boundary tests.
- [x] Guided preventive/corrective workflow, review, mouse/touch signature and protected storage.
- [x] Atomic/idempotent maintenance, valve dates, one-unit kit consumption and pending-consumption policy.
- [x] Per-valve timeline and searchable global maintenance history.
- [x] Browser E2E covers valve, photo, maintenance, stock and resulting UI state.
- [ ] Add valve deletion only if retention policy confirms it is required.
- [ ] Add server-side image-dimension validation if installations accept untrusted large-dimension images.

## Orders, stock, purchasing and suppliers

- [x] Canonical order priority/status enums, tested transitions, audited deletion and planning list.
- [x] Zone criticality is derived from canonical valve health.
- [x] Restock create/review/delete with session actors, reviewer and timestamp.
- [x] Non-negative stock, normalized references, movements, reasons and safe zero-only deletion.
- [x] Supplier create/edit/delete with admin RBAC and email/HTTP(S) validation.
- [x] Grouped purchase plan, editable quantities, CSV export and localized email output.
- [x] Concurrent transaction and idempotent retry tests cover the supported single-writer adapter.

## Reports, notifications and localization

- [x] Backend datasets for overview, zones, critical valves and all recent maintenance rows.
- [x] Paginated localized PDF with metadata, continuation and page numbering.
- [x] Deduplicated critical-valve, low-stock, open-order and runtime-health notifications.
- [x] Typed PT/EN/ES/FR catalogs for all application-owned visible and error text.
- [x] Locale persistence and localized date/number formatting.
- [x] Runtime-derived local read/write, degraded and sync-disabled UX; no `navigator.onLine` claim.
- [x] PWA intentionally disabled; no stale Vite service worker controls authenticated operational data.

## Migration and release

- [x] Separate deterministic migration CLI with dry-run, batch/checksum idempotency and JSON/Markdown reports.
- [x] Legacy aliases, entity counts, relationship warnings, audit history and photo metadata import.
- [x] Passwords excluded; fixture and repeated-import tests pass.
- [ ] Reconcile counts against the real production export supplied at cutover.
- [ ] Transfer source photo binaries after receiving the cutover archive and mapping source references.
- [x] Rust tests, Biome, TypeScript, Next build, dependency audits and committed E2E pass.
- [x] Production-volume query/audit pagination and concurrent storage tests pass.
- [ ] Execute target-environment disk/network/read-only failure drills and restore test.
- [ ] Complete external items above before production sign-off.
