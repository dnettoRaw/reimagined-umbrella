# PROEXEL architecture - CONCLUÍDO 100%

Verified against the implementation on 2026-08-15.

## Boundaries

PROEXEL is an AppCore consumer with its own Rust workspace and Next.js UI. Product
concepts remain in `proexel/`; the generic runtime contains no PROEXEL entities.

Dependency direction:

```text
proexel-domain
      ^
proexel-application
      ^
proexel-infrastructure
      ^
proexel-service -> public AppCore contracts/runtime host

apps/web -> authenticated HTTP capability proxy -> proexel-service
```

The boundary test rejects dependencies from application/domain into AppCore or
the web layer.

## Domain model

```text
Machine
  -> MachineItem[] (stable functional positions)
       -> ItemCategory
            -> typed custom fields
            -> versioned structured MaintenanceGuide
       -> InstalledComponent (replaceable physical unit)
       -> ReplacementSpecification
       -> MachineItemReplacement[]

ServiceOrder
  -> immutable MachineSnapshot
  -> ServiceOrderTask[]
       -> immutable MachineItemSnapshot + ItemCategorySnapshot + guide photos
       -> ItemInspection
            -> structured step results, findings and evidence photos
```

`ItemCategory` is dynamic and has no rigid component-type enum. Custom field and
guide step values are validated by type. `ComplexityLevel` is a domain value
restricted to 1..5. Machine status is deterministically derived from active item
statuses.

`MachineItem` represents a functional position. `InstalledComponent` represents
the current physical unit, so replacement preserves the position, old/new serial
numbers, actor, reason and timestamp.

## Commands and consistency

The service accepts explicit AppCore capabilities. Every write passes through
the application state transaction, domain validation, RBAC, idempotency receipt
and semantic audit event before the infrastructure adapter atomically replaces
the JSON state file.

Important invariants include:

- category must be active when adding an item;
- item must belong to the selected machine;
- operator level must cover task complexity when assigning, starting or completing;
- required guide steps and photo evidence must be present;
- measurement units must match the snapshot;
- service orders cannot complete with pending tasks;
- guide photos frozen in an order cannot be deleted;
- machine status is refreshed after item/inspection state changes.

## Storage and migration

Canonical state uses schema version 2. Existing schema-v1 files are decoded,
backed up as `.schema-v1.json.bak`, transformed in place and persisted as v2.
The separate `proexel-migration` CLI imports legacy exports deterministically and
maps each legacy valve to a `MachineItem` in a reusable Valve category. Those
legacy DTO names exist only in migration code.

The supported deployment is standalone local read/write. Remote sync is not
configured and the UI does not claim remote success or offline synchronization.

## Read projections

Queries expose overview, categories, paginated machines with items/photos/history,
orders, inspections, operators, stock, purchasing, suppliers, audit and reports.
Order and inspection projections retain immutable category/guide context.

## Web boundary

Next.js owns authentication, signed `HttpOnly` sessions, route authorization,
server-only AppCore tokens and protected attachment transport. It never reads or
writes canonical state directly. The Rust service repeats authorization and all
domain checks.

Visible product text and service errors are typed and present in Portuguese,
English, Spanish and French. Portuguese is the fallback locale.

## Verification

- Rust workspace tests cover policies, commands, storage, migration and query projections.
- Clippy runs for all targets with warnings denied.
- Biome, TypeScript and the Next.js production build pass.
- Playwright runs a real isolated AppCore stack and completes the machine workflow.
- `npm audit --audit-level=high` reports zero vulnerabilities.
