# PROEXEL rebuild implementation status

Status verified against code and automated gates on 2026-08-15. A prompt title
contains `CONCLUÍDO 100%` only when every applicable requirement is delivered.

## Completed prompts

| Prompt | Status | Evidence |
|---|---|---|
| 01 Discovery | 100% | Behavior, data, RBAC and AppCore maps plus parity checklist |
| 02 Architecture | 100% | Independent Rust/Next workspace, public AppCore boundary, manifests and boundary test |
| 03 Bootstrap | 100% | Real health/query slice, durable empty state, PROEXEL shell and local launcher |
| 04 Domain/storage | 100% | Typed schema v1, atomic/idempotent transactions, maintenance/stock policies and tests |
| 05 Auth/RBAC/audit | 100% | Scrypt password/PIN, versioned sessions, user administration, four-role enforcement and redacted audit |
| 08 Operations | 100% | Orders/schedule, canonical priority, stock/movements, restock, suppliers and purchase output |
| 09 Reports/i18n/offline | 100% | Paginated PDF, PT/EN/ES/FR, local/degraded status and deduplicated notifications |

The supported topology is standalone local read/write. Remote sync is explicitly
disabled, so pending/conflict states do not exist in this deployment and are not
presented as successful.

## Partially open prompts

| Prompt | Implemented | Remaining reason the title is not 100% |
|---|---|---|
| 00 Master | All definition-of-done engineering gates for standalone | Master scope includes the still-open prompt 10/11 cutover and resilience evidence |
| 06 UI | All product routes, responsive workflows, filters, pagination and audit detail | Tables/forms do not uniformly use the exact TanStack/RHF stack requested by the prompt |
| 07 Valves/maintenance | Create/edit/detail, stable-ID photos, guided signed maintenance, stock and E2E | Valve deletion and server-side image-dimension validation are not implemented |
| 10 Migration | Deterministic CLI, aliases, dry-run, checksums, reports and metadata links | Requires a real production export and source photo binaries to validate/transfer |
| 11 Release | Unit/integration/E2E, volume/concurrency, lint/type/build and dependency audits | Full disk/network/read-only/sync fault injection remains external/open |

## Verified gates

- Rust workspace: 28 tests, including domain boundaries, identities, storage, migration,
  concurrency and production-volume pagination.
- Browser E2E: admin, technician and chief operational flow, signed maintenance,
  attachment, stock, order, restock approval, PDF and mobile overflow.
- Web: Biome, TypeScript, Next.js production build and npm audit.
- Dependencies: npm and Rust audits are release gates.

The exact remaining gates are maintained in
[`functional-parity-checklist.md`](functional-parity-checklist.md).
