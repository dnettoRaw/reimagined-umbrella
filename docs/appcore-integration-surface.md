# AppCore integration surface for PROEXEL

Status: implemented AppCore consumer surface, 2026-08-13.

## Located runtime and templates

- AppCore path: `core/AppCore-Runtime`
- AppCore submodule commit: `12cfc5d3e78f4c7918b335907aab27c2205e710f`
- AppCore version marker: `v1.0.1-rc.8-10-g12cfc5d`
- Admin dashboard template: `admin-dashboard-templ-main.zip` and `/Users/dan/Downloads/admin-dashboard-templ-main`
- Legacy PROEXEL: `PROEXEL-main.zip`

## Boundary

AppCore is product-independent infrastructure. PROEXEL must live outside
`core/AppCore-Runtime/crates/*` and consume only stable public contracts.

PROEXEL owns:

- domain aggregates and policies;
- application commands and queries;
- schema, migrations and import rules;
- RBAC permissions and business authorization;
- UI and localization;
- report semantics;
- conflict/ownership decisions for business data.

AppCore can provide:

- manifest validation and application host facade;
- runtime lifecycle and health/status;
- generic command/query transport;
- storage provider contracts and bounded local file storage;
- idempotency support and command dispatch primitives;
- audit/observation infrastructure;
- signed token/security primitives;
- leader-to-follower sync infrastructure;
- provider selection via deployment manifest;
- update/supervisor/runtime operations.

## AppCore documents read

| Document | Key implication |
|---|---|
| `core/AppCore-Runtime/docs/architecture.md` | AppCore is contracts-first, local-first infrastructure; business handlers are registered by the application. |
| `core/AppCore-Runtime/APPLICATION_MANIFEST.md` | PROEXEL must provide `ApplicationManifestV1` with functional capabilities and no secrets/paths. |
| `core/AppCore-Runtime/DEPLOYMENT_MANIFEST.md` | Deployment owns paths, provider choices, endpoints, TLS refs and secrets. No business behavior belongs there. |
| `core/AppCore-Runtime/PROVIDER_MODEL.md` | Providers are selected by deployment manifest; missing/invalid providers fail explicitly; no silent fallback. |
| `crates/appcore-api/README.md` | Stable HTTP routes include health, status and command/query V1; product REST resources do not belong in AppCore. |
| `crates/appcore-contracts/README.md` | Stable manifests, modes, capabilities and policies; no I/O or business concepts. |
| `crates/appcore-types/README.md` | Use validated IDs, runtime identity and trace context at contract boundaries. |
| `crates/appcore-storage/README.md` | Application schemas remain app-owned; unsupported transactions fail explicitly. |
| `crates/appcore-sync/README.md` | Sync is leader-to-follower replication, not RAFT/multi-master/domain conflict resolution. |
| `crates/appcore-security/README.md` | Tokens/secrets/policy contracts are reusable; domain authorization remains external. |
| `crates/appcore-core/README.md` | Generic lifecycle, dispatch, audit and idempotency; no product domain. |
| `crates/appcore-bin/README.md` | New applications implement `appcore_bin::application::Application` and call `run_application`. |

## Runtime capabilities

AppCore validates manifest capabilities against exact registered command/query
names. PROEXEL can still group behavior conceptually by domain prefix, but the
runtime manifest must declare concrete operations.

Validated command capabilities:

- `proexel.valves.create`
- `proexel.valves.update`
- `proexel.valves.add_photo`
- `proexel.valves.delete_photo`
- `proexel.maintenance.register`
- `proexel.orders.create`
- `proexel.orders.change_status`
- `proexel.orders.delete`
- `proexel.purchasing.create_restock_request`
- `proexel.purchasing.review_restock_request`
- `proexel.purchasing.delete_restock_request`
- `proexel.stock.adjust`
- `proexel.stock.upsert_item`
- `proexel.stock.delete_item`
- `proexel.suppliers.create`
- `proexel.suppliers.update`
- `proexel.suppliers.delete`
- `proexel.admin.users.create`
- `proexel.admin.users.update`
- `proexel.admin.users.reset_credentials`

Validated query capabilities:

- `proexel.overview.get`
- `proexel.valves.list`
- `proexel.maintenance.list`
- `proexel.orders.list`
- `proexel.purchasing.list_restock_requests`
- `proexel.stock.list`
- `proexel.suppliers.list`
- `proexel.audit.list`
- `proexel.reports.get`
- `proexel.admin.users.list`
- `proexel.identity.resolve`

These names are application-owned. They must not use reserved namespaces
`appcore.*`, `runtime.*` or `infrastructure.*`.

`proexel.identity.resolve` is the only pre-session query. Its scoped bearer token
is held exclusively by Next.js on the protected service network. It returns one
identity to the server authentication adapter; hashes are never returned by the
admin list API or browser UI.

## Storage approach

- Keep domain schema versioned in PROEXEL migrations.
- Use AppCore storage as generic provider boundary where appropriate, not as the owner of PROEXEL tables.
- Writes that may be retried must carry idempotency keys.
- The standalone canonical state is `target/runtime/storage/proexel-state-v1.json`
  relative to the deployment manifest, unless `PROEXEL_DATA_FILE` overrides it.
- State commits use a locked clone/validate/write transaction and atomic file
  replacement; schema ownership remains inside PROEXEL.
- Maintenance stock debit must be one atomic application transaction. If the selected storage provider cannot guarantee the transaction, the application must fail explicitly rather than pretending success.
- Supabase is a legacy adapter/import source unless a later ADR selects it as an optional provider.

## Audit and observability

- Application command handling should emit audit events centrally.
- The local audit event is part of the reliable application write path.
- Secondary audit sinks are non-blocking; failures are observable and retryable, but do not erase local audit.
- Trace/correlation ID should use AppCore trace types at boundaries when possible.

## Sync and conflict policy

AppCore sync can replicate infrastructure logs leader-to-follower. PROEXEL still needs domain decisions for:

- who owns writes in offline/degraded modes;
- how service-order assignment interacts with local operation;
- whether stock adjustments require leadership;
- conflict resolution for mutable fields such as TAG and supplier details;
- replay idempotency for maintenance stock debit.

## Security and RBAC

AppCore security provides reusable token/secret primitives. PROEXEL must define:

- roles and permissions;
- login/auth adapter behavior;
- command/query authorization;
- denial audit behavior;
- UI state derived from permissions.

The web adapter reads the actor and role only from an HMAC-signed, `HttpOnly`
session cookie. Passwords and optional PINs are verified server-side with scrypt
hashes resolved through a server-only scoped AppCore query. The proxy protects dashboard and
product API routes, pages enforce read permissions, the command proxy checks the
same permission matrix, and the application handler independently enforces RBAC.

Legacy auth details replaced:

- client SHA-256 password hashing with salt `_proexel_salt_2026`;
- localStorage session `mp_session` lasting 8h;
- localStorage rate limit `mp_login_attempts`;
- Supabase SECURITY DEFINER RPC `app_login`.

The local launcher generates a random administrator password and session secret
for each run. The environment user directory is imported only as first-run seed;
subsequent management is capability-protected and audited in canonical state.

## Admin dashboard template surface

Useful existing pieces:

- Next.js 16 App Router with TypeScript.
- shadcn UI components in `src/components/ui`.
- Sidebar/header layout under `src/app/(main)/dashboard`.
- Navigation config in `src/navigation/sidebar/sidebar-items.ts`.
- TanStack Table dependency, React Hook Form, Zod, Recharts and date components.
- Theme/layout preferences via cookies/local client preference stores.

Template content to remove/replace:

- demo dashboards: default, CRM, finance, analytics, productivity, ecommerce, academy, logistics;
- demo mail/chat/calendar/kanban/invoice/users routes unless reworked for PROEXEL;
- legacy dashboard group;
- marketing/template copy and external GitHub links in product chrome.
