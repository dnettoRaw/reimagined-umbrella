# PROEXEL local operations

## Storage ownership

The AppCore-hosted service is the only writer of canonical PROEXEL state. Next.js
uses scoped command/query tokens and never reads the state file directly. The
JSON adapter writes a complete temporary file, flushes it, and renames it over
the previous version while holding the application transaction lock.

## Backup and restore

Follow [`backup-restore.md`](backup-restore.md) for the cold-copy, checksum,
validation, restore, and rollback procedure. The deployment manifest declares
`proexel/apps/service/target/runtime/backups`, but the current PROEXEL adapter
does not schedule or create application backups automatically.

## Authentication

`dev-stack.sh` prints a random, one-run administrator credential. Deployed
installations must set a random `PROEXEL_SESSION_SECRET` of at least 32
characters and a server-only `PROEXEL_AUTH_USERS` JSON array. Password hashes use
`scrypt$salt$hex`; plaintext passwords must not be stored in that configuration.
Sessions are signed, `HttpOnly`, `SameSite=Strict`, and expire after eight hours
unless the operator explicitly selects the 30-day option. Five failed attempts
from one forwarded client address trigger a 15-minute lockout.

## Runtime states

- Connected: AppCore health and capability queries are accepted.
- Unavailable: the web UI presents empty/error states and does not claim writes
  succeeded.
- Local read/write: commands commit to the local durable state before success.
- Sync pending/conflict: reserved for a future configured sync adapter; the
  standalone deployment does not claim remote synchronization.

## Troubleshooting

Use [`troubleshooting.md`](troubleshooting.md) for startup, authentication,
token, storage, migration, locale, and build failures. Deployment topology and
upgrade/rollback steps are in [`deployment.md`](deployment.md).
