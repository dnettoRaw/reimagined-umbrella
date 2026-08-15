# PROEXEL troubleshooting

Start with the narrowest failing boundary: browser -> Next.js session -> Next.js
API adapter -> AppCore transport -> PROEXEL application -> state file.

## Quick diagnostics

From the repository root in a local deployment:

```bash
curl -i http://127.0.0.1:39400/v1/health
test -r proexel/apps/service/target/runtime/storage/proexel-state-v1.json
jq '.schema_version' proexel/apps/service/target/runtime/storage/proexel-state-v1.json
```

Then run:

```bash
cargo test --manifest-path proexel/Cargo.toml --workspace
cd proexel/apps/web
npm run check
npx tsc --noEmit
npm run build
```

## Symptoms

### Login page loads but authentication says it is not configured

- Confirm `PROEXEL_SESSION_SECRET` is visible to the Next.js process and is at
  least 32 characters.
- On first boot, confirm `PROEXEL_AUTH_USERS` contains a valid active admin; on
  later boots, verify canonical state contains at least one active account.
- Restart Next.js after changing its environment.
- Do not place these variables in browser-side environment files.

### Credentials are always rejected

- Compare the submitted email case-insensitively with the configured record.
- Confirm the hash is exactly `scrypt$salt$hex` and was generated with the same
  password and salt.
- The password must contain at least eight characters.
- A PIN must contain only 4 to 8 digits and must have been configured separately.
- Confirm the account is active. Role/status/credential changes intentionally
  invalidate existing sessions.
- Five failures from one forwarded address lock that address for 15 minutes in
  the current Next.js process. Restarting only to bypass a production lockout is
  not an approved response; investigate the client and proxy first.

### Redirect loop between login and dashboard

- Confirm all Next.js instances use the same `PROEXEL_SESSION_SECRET`.
- Clear the `proexel_session` cookie after a deliberate secret rotation.
- Confirm proxy clock and host clock agree; session expiration uses epoch
  milliseconds.
- Verify the public origin and HTTPS proxy preserve cookies.

### Dashboard shows the service as unavailable

- Call `/v1/health` directly from the Next.js host.
- Confirm `PROEXEL_SERVICE_URL` points to the Rust listener.
- Confirm loopback/container networking matches the deployment topology.
- Inspect the Rust process startup error before assuming empty data.

### Reads are empty but health succeeds

- Confirm the signed-in role has the query permission in `rbac-matrix.md`.
- Confirm `PROEXEL_SERVICE_TOKENS` contains the exact query capability.
- Confirm the Rust service is reading the intended `PROEXEL_DATA_FILE`.
- An empty state is valid on a new installation; compare the file/entity counts.

### Command returns `invalid bearer token` or unauthorized

- Scoped AppCore tokens may have expired. The local default lifetime is three
  hours.
- Restart `scripts/dev-stack.sh` or regenerate the complete capability map.
- Confirm a command token was generated for a command and a query token for a
  query.
- Never substitute a token from another deployment manifest.

### `security secret expired` during startup

The file-backed bootstrap secret is short-lived. Build AppCore first, rotate the
secret immediately before startup and generate capability tokens in the same
controlled startup sequence. The current local launcher follows this order.

### `storage_decode_failed`

- Stop the Rust service immediately; do not let another writer replace files.
- Preserve the failing state and its `.tmp` file for diagnosis.
- Validate JSON separately.
- Restore a checksum-verified backup using [backup-restore.md](backup-restore.md).
- Do not manually remove unknown fields or idempotency receipts.

### `storage_write_failed`, `storage_sync_failed` or `storage_commit_failed`

- Check disk space, inode availability, ownership and directory permissions.
- Confirm the temporary and canonical files are on the same filesystem so rename
  remains atomic.
- Confirm only one service process writes the file.
- After remediation, restart and verify a controlled write survives restart.

### Port 3000 or 39400 is already in use

Find and stop the existing stack or set `PROEXEL_WEB_PORT` and an alternate
deployment manifest. The E2E runner intentionally uses 3010/39410 and isolated
state/build directories. Do not run two Rust writers against one state file.

### Photo or signature returns 404

- Confirm `PROEXEL_ATTACHMENTS_DIR` is the same absolute path used at upload.
- Confirm the Next.js account can read files and traverse directories.
- Restore attachments from the same backup consistency point as canonical state.
- Do not edit `blob_ref` values or expose the attachment directory publicly.

### Language changes only after navigation

The current selector writes `proexel_locale` and reloads the current route. If
the old language remains, check that cookies are enabled, the cookie path is
`/`, and a proxy is not stripping `Set-Cookie`/request cookies. Valid values are
`pt`, `en`, `es` and `fr`.

### Migration imports zero records

- Check whether the same batch/checksum already ran; reruns are intentionally
  idempotent.
- Review JSON and Markdown reports for duplicate identities and unresolved
  links.
- Confirm top-level collection names follow `migration-runbook.md`.
- Run with a new isolated state path during diagnosis; do not mutate production
  state repeatedly.

### UI build fails after dependency changes

Use `npm ci`, not an unreviewed lockfile rewrite. Run Biome, TypeScript and the
production build separately to identify the failing gate. Keep Node/npm aligned
with the environment that produced the committed lockfile.

Run the isolated browser workflow from the repository root with
`./proexel/scripts/e2e.sh`; it creates ephemeral credentials and storage.

## Escalation evidence

Collect the application/runtime revisions, redacted manifests, process start
time, failing route/capability, HTTP status, Rust/Next.js logs, state schema
version and last known successful operation. Never include bearer tokens,
session cookies, plaintext passwords or the full `PROEXEL_AUTH_USERS` value.
