# PROEXEL configuration - CONCLUÍDO 100%

Configuration is split between the AppCore application manifest, the deployment
manifest and server-only environment variables. Never place bearer tokens,
session secrets or password hashes in client-exposed `NEXT_PUBLIC_*` variables.

## Requirements

- Rust toolchain compatible with both workspace lockfiles.
- Node.js and npm compatible with `proexel/apps/web/package-lock.json`.
- OpenSSL command-line tools for `scripts/dev-stack.sh`.
- The initialized `core/AppCore-Runtime` submodule.

Install the web dependencies with `npm ci` from `proexel/apps/web`.

## AppCore manifests

`proexel/apps/service/application.toml` owns the application identity,
capability allowlist, health policy and schema metadata. Command/query names in
that file must remain aligned with
`proexel/crates/proexel-application/src/commands.rs`.

`proexel/apps/service/deployment.local.toml` selects standalone mode, file
storage, runtime paths and loopback HTTP at `127.0.0.1:39400`. It is a local
manifest: TLS is disabled and the listener must not be exposed directly to an
untrusted network.

## Environment variables

| Variable | Process | Required | Meaning |
|---|---|---:|---|
| `APPCORE_APPLICATION_MANIFEST` | Rust service | Yes | Path to `application.toml` |
| `APPCORE_DEPLOYMENT_MANIFEST` | Rust service | Yes | Path to the selected deployment manifest |
| `PROEXEL_DATA_FILE` | Rust service | No | Overrides the canonical JSON state path |
| `PROEXEL_ATTACHMENTS_DIR` | Next.js | No | Protected machine/component/guide/inspection photo root; defaults beside service runtime storage |
| `PROEXEL_SERVICE_URL` | Next.js | Yes | AppCore service base URL, normally `http://127.0.0.1:39400` |
| `PROEXEL_SERVICE_TOKENS` | Next.js | Recommended | JSON map from capability name to scoped bearer token |
| `PROEXEL_SERVICE_TOKEN` | Next.js | Compatibility only | Generic fallback token when no scoped map entry exists |
| `PROEXEL_SESSION_SECRET` | Next.js | Yes | Random HMAC secret, at least 32 characters |
| `PROEXEL_AUTH_USERS` | Rust service | Bootstrap only | Initial users imported only when canonical user state is empty |
| `PROEXEL_DEV_TTL_MS` | Dev launcher | No | Scoped token lifetime; defaults to 10,800,000 ms (3 hours) |
| `PROEXEL_WEB_PORT` | Dev/E2E launcher | No | Next.js port; defaults to `3000` |
| `PROEXEL_DEPLOYMENT_MANIFEST` | Dev/E2E launcher | No | Alternate AppCore deployment manifest |
| `PROEXEL_NEXT_DIST_DIR` | Next.js | No | Alternate build directory for isolated test runs |

The checked-in starting template is `proexel/.env.example`; the table above is
the complete documented surface. Environment files must remain untracked and
readable only by the service account.

## User record format

`PROEXEL_AUTH_USERS` is a one-time seed array with this shape:

```json
[
  {
    "id": "operator-001",
    "email": "operator@example.invalid",
    "name": "Operator name",
    "role": "tecnico",
    "password_hash": "scrypt$<salt-hex>$<derived-key-hex>",
    "pin_hash": "scrypt$<salt-hex>$<derived-key-hex>",
    "active": true
  }
]
```

Allowed roles are `admin`, `chefe`, `compras` and `tecnico`. IDs and emails
must be unique. Passwords have 8 to 128 characters and optional PINs have 4 to
8 digits. Plaintext credentials never enter the Rust command or audit payload.

The local launcher creates an ephemeral administrator and prints its random
password. A production bootstrap must seed at least one active administrator,
then remove the environment value. Later users, roles, activation and
credentials are managed in **Users and access**; disabling or changing an
account increments its authentication version and invalidates current sessions.

## Scoped tokens

The Next.js process requires one AppCore token per command/query capability.
`scripts/dev-stack.sh` generates the complete JSON map and keeps it only in the
process environment. Tokens expire; restart the local stack after the configured
three-hour default or regenerate the map with an appropriate operational
lifetime.

Production token generation must happen during controlled startup using the
selected deployment manifest. Do not copy a development token into a committed
environment file and do not expose it to browser JavaScript.

## Locale and UI preferences

Locale is selected by the browser and persisted as `proexel_locale` with one of
`pt`, `en`, `es` or `fr`. Theme/layout preferences and sidebar state use their
own browser cookies/storage and are not canonical operational data.

## Configuration validation

Before accepting traffic:

1. Validate both manifests with the AppCore tooling used by the target release.
2. Confirm the runtime secret file exists with restrictive permissions.
3. Confirm `PROEXEL_SESSION_SECRET` is present and at least 32 characters.
4. On first bootstrap, confirm the seed contains an active admin and valid scrypt hash.
5. Confirm every capability used by the web adapter has a scoped token.
6. Confirm state, attachment and backup directories are owned by the runtime account.
7. Call `/v1/health` and perform an authenticated overview query.
