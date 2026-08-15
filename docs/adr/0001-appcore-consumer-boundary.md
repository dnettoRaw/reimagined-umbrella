# ADR 0001: AppCore consumer boundary

Status: accepted, 2026-08-13.

## Context

PROEXEL is being rebuilt from a React/Vite/Supabase legacy application. AppCore
is present as `core/AppCore-Runtime` and is a generic local-first runtime. The
legacy product rules must not be moved into AppCore crates.

## Decision

PROEXEL will be implemented as an external consumer workspace under `proexel/`.
AppCore remains an infrastructure/runtime dependency consumed by `proexel/apps/service`.

The application is split into:

- `proexel-domain`: aggregates, value objects, enums and policies.
- `proexel-application`: commands, queries and RBAC decisions.
- `proexel-infrastructure`: storage, audit, sync and external adapters.
- `proexel-migration`: repeatable import from the legacy Supabase/static data.
- `apps/service`: AppCore-hosted composition root.
- `apps/web`: Next.js admin dashboard, session boundary, and server-side
  transport adapter.

## AppCore responsibilities

- Manifest and deployment validation.
- Runtime lifecycle, health and status.
- Generic command/query transport.
- Provider composition.
- Generic storage boundary.
- Idempotency and command dispatch primitives.
- Audit/observation infrastructure.
- Security primitives and secret references.
- Leader-to-follower sync infrastructure.
- Supervisor/update/runtime operations.

## PROEXEL responsibilities

- Domain schema, migrations and local data model.
- Valve, maintenance, stock, order, purchasing, supplier, report and audit rules.
- RBAC permissions and command/query authorization.
- Idempotent maintenance stock debit.
- Conflict and ownership policy for local-first operation.
- Legacy import and compatibility aliases.
- UI routes, forms, tables, reports and i18n.

## Storage

PROEXEL owns schema versioning and domain migrations. AppCore storage providers
may be used as the generic persistence boundary, but they do not define PROEXEL
tables or business transactions.

Maintenance registration with kit change must be an application transaction:
insert maintenance record, debit stock when applicable, and persist local audit.
If a provider cannot satisfy that transaction, the command fails explicitly.

## Audit and observability

Audit is centralized in the application command pipeline. The local audit event
is reliable application state. Secondary export/sink failure is observable and
non-blocking.

## Sync

AppCore sync is infrastructure replication, not domain conflict resolution.
PROEXEL must decide write ownership, leadership requirements and replay
idempotency per command.

## Security

AppCore security primitives do not replace PROEXEL authorization. RBAC is
enforced by application command/query handlers. UI visibility is only a
convenience layer.

## Consequences

- `core/AppCore-Runtime/crates/*` remains untouched by PROEXEL rules.
- The new domain can be tested without AppCore or UI dependencies.
- Supabase is treated as a legacy adapter/import source unless a later ADR
  explicitly selects it as a deployment adapter.
