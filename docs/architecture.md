# PROEXEL architecture

Status: implemented baseline, verified against the code on 2026-08-15.

## System context

PROEXEL is an external consumer of AppCore Runtime. Product rules live in the
`proexel/` workspace; no valve, maintenance, stock, purchasing or supplier rule
belongs in `core/AppCore-Runtime/crates`.

The running standalone system has two processes:

1. `proexel-service`, a Rust application hosted by AppCore on
   `127.0.0.1:39400` by the local manifest.
2. `proexel-web`, a Next.js server that owns the browser session, renders the UI
   and calls AppCore command/query routes with server-only scoped tokens.

The browser never receives AppCore bearer tokens and never opens the canonical
state file.

## Component boundaries

| Component | Responsibility | Must not own |
|---|---|---|
| `proexel-domain` | Entities, enums, normalization, maintenance and stock policies | HTTP, sessions, files, React |
| `proexel-application` | Commands, queries, RBAC, idempotency, audit semantics and state transitions | AppCore internals, Next.js |
| `proexel-infrastructure` | Durable JSON adapter and transaction boundary | Product authorization decisions |
| `proexel-migration` | Deterministic legacy import, aliases, warnings and reports | Runtime compatibility hacks |
| `apps/service` | AppCore composition root, capability registration and report datasets | Browser authentication |
| `apps/web` | Session handling, route protection, localized UI and server-side transport adapter | Canonical business state |
| AppCore Runtime | Lifecycle, manifests, scoped transport authorization and health | PROEXEL domain rules |

The dependency direction is domain <- application <- infrastructure/service.
`proexel-application/tests/boundaries.rs` checks that the application layer does
not depend on AppCore or the web application.

## Command flow

1. The browser sends a request to a same-origin `/api/proexel/*` route.
2. The Next proxy verifies the signed `proexel_session` cookie.
3. The server adapter resolves the capability token and sends an AppCore
   `/v1/command` envelope.
4. AppCore validates the scoped token and dispatches the named capability.
5. `ApplicationState::execute` validates the actor and permission, detects an
   idempotent replay, applies the domain transition and creates the local audit
   event.
6. `JsonFileStore::transact` persists the complete candidate state before it
   replaces the in-memory state.
7. The accepted result is returned to Next.js; the UI refreshes its query data.

All registered writes require an idempotency key. `processed_commands` stores
the resulting receipt, so replay does not repeat stock consumption or create a
second aggregate.

## Query flow

The browser request is authenticated by Next.js and forwarded with a scoped
query token. The Rust service reads one state snapshot and produces DTO-shaped
JSON. Maintenance health and report criticality are calculated on the service
side from the domain policy; the UI only presents the contracted result.

Queries include overview, valves, maintenance, service orders, restock requests,
stock, suppliers, audit and reports. Valve and audit queries filter and paginate
in the Rust service; audit supports actor, operation, entity, date and free-text
criteria. Valve detail includes stable-ID photo metadata and its timeline.

## Persistence and transactions

The canonical standalone state is schema version 1. By default it is stored at:

```text
proexel/apps/service/target/runtime/storage/proexel-state-v1.json
```

`PROEXEL_DATA_FILE` overrides this path. A write clones the current state,
applies the operation, serializes to `*.json.tmp`, calls `sync_all`, then renames
the temporary file over the canonical file while holding a process-local mutex.
An operation or persistence failure leaves the previous in-memory state active.

This gives atomic application transitions for the current single-process,
single-writer deployment. It is not a multi-process database and must not be
mounted for concurrent writers.

## Authentication and authorization

User accounts are canonical Rust state and can be managed only through
`admin.users.manage`. Next.js resolves one identity through a dedicated,
server-only AppCore capability, verifies scrypt password/PIN hashes and issues
an HMAC-SHA-256 signed, `HttpOnly`, `SameSite=Strict` cookie. An authentication
version immediately invalidates sessions after role, status or credential
changes. The default session lasts eight hours; the explicit remember option
lasts 30 days. Five failures per forwarded client address cause a 15-minute
in-process lockout.

Authorization is repeated at two application-facing boundaries:

- Next server pages and API adapters enforce route/action permissions for UX
  and early rejection.
- `proexel-application` enforces the canonical permission for every command and
  query capability.

The implemented matrix is documented in [rbac-matrix.md](rbac-matrix.md).

## Internationalization

The web application supports `pt`, `en`, `es` and `fr`. A typed catalog is used
by server and client components, the selected locale is persisted in the
`proexel_locale` cookie, and dates/numbers use locale-specific `Intl` formats.
User-entered operational data is not machine-translated.

## Attachments and reports

Photo and signature bytes live outside canonical JSON in a protected local file
adapter selected by `PROEXEL_ATTACHMENTS_DIR`. Next.js validates authentication,
RBAC, media type, magic bytes, size and safe generated paths; canonical metadata
stores immutable valve IDs and opaque references. PDF reports use backend report
datasets and paginate every row without recalculating health rules.

## Runtime and sync status

The checked-in deployment is `standalone`, local read/write and file-backed.
No outbox, remote sync provider or conflict resolver is configured. The UI may
report the AppCore service as connected or unavailable, but it must not claim
that remote synchronization succeeded.

## Current limitations

- No remote sync/outbox is configured for the supported standalone deployment.
- Initial administrator seeding remains a controlled deployment step; later
  accounts are managed in the admin UI.
- Shared rate limiting is required only if the web tier is horizontally scaled.
- The local attachment adapter does not generate thumbnails or validate decoded
  image dimensions.
- The JSON adapter remains single writer and is not a distributed database.

See [functional-parity-checklist.md](functional-parity-checklist.md) for the
feature-level status.
