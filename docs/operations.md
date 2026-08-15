# PROEXEL local operations

## Storage ownership

The AppCore-hosted service is the only writer of canonical PROEXEL state. Next.js
uses scoped command/query tokens and never reads the state file directly. The
JSON adapter writes a complete temporary file, flushes it, and renames it over
the previous version while holding the application transaction lock.

Next.js is the only writer of protected photo/signature files under
`PROEXEL_ATTACHMENTS_DIR`. Attachment metadata remains canonical Rust state, so
state and files must be backed up and restored together.

## Backup and restore

Follow [`backup-restore.md`](backup-restore.md) for the cold-copy, checksum,
validation, restore, and rollback procedure. The deployment manifest declares
`proexel/apps/service/target/runtime/backups`, but the current PROEXEL adapter
does not schedule or create application backups automatically.

## Authentication

`dev-stack.sh` prints a random, one-run administrator credential. Deployed
installations must set a random `PROEXEL_SESSION_SECRET` of at least 32
characters. `PROEXEL_AUTH_USERS` seeds the first active administrator only when
canonical user state is empty. Password/PIN hashes use `scrypt$salt$hex`;
plaintext credentials are accepted only by the protected Next.js API and are
never persisted or audited.
Sessions are signed, `HttpOnly`, `SameSite=Strict`, and expire after eight hours
unless the operator explicitly selects the 30-day option. Five failed attempts
from one forwarded client address trigger a 15-minute lockout.
Role, activation and credential changes increment `auth_version`, invalidating
all existing sessions for that user.

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
