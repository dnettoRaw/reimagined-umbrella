# PROEXEL local operations

## Storage ownership

The AppCore-hosted service is the only writer of canonical PROEXEL state. Next.js
uses scoped command/query tokens and never reads the state file directly. The
JSON adapter writes a complete temporary file, flushes it, and renames it over
the previous version while holding the application transaction lock.

## Backup and restore

For a consistent manual backup, stop `dev-stack.sh`, copy
`proexel/apps/service/target/runtime/storage/proexel-state-v1.json` to a dated,
access-controlled location, then restart. Restore only while the service is
stopped. The AppCore deployment also provides its configured backup directory at
`proexel/apps/service/target/runtime/backups`.

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

- `security secret expired` during startup: use the current launcher, which
  prebuilds AppCore and emits all scoped tokens inside the 60-second bootstrap
  secret window.
- `invalid bearer token`: regenerate the scoped token map by restarting the stack.
- `storage_decode_failed`: stop the service and restore a known-good backup.
- Port already in use: stop the existing PROEXEL dev process before launching a
  second stack.
- Web reads show `unavailable`: verify `/v1/health`, `PROEXEL_SERVICE_URL`, and
  the capability token map.
