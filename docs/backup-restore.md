# PROEXEL backup and restore

The standalone deployment stores canonical metadata in one schema-versioned
JSON file and photo/signature bytes in the attachment directory. Use cold
backups: stop Rust and Next.js before copying either consistency set.

## Paths

The default canonical file is:

```text
proexel/apps/service/target/runtime/storage/proexel-state-v1.json
```

If `PROEXEL_DATA_FILE` is set, that value is authoritative. The deployment
manifest also declares `target/runtime/backups`, but the PROEXEL adapter does
not currently schedule application backups automatically. Operators must run
and monitor the procedure below or provide equivalent platform automation.

The state file contains operational and audit data. Treat backups as sensitive:
restrict access, encrypt external copies and define retention according to the
installation's policy.

The attachment root defaults to
`proexel/apps/service/target/runtime/attachments` and is overridden by
`PROEXEL_ATTACHMENTS_DIR`. Back up the complete directory with the state file;
metadata without its referenced binary is incomplete.

## Create a consistent backup

1. Announce a write interruption and stop the Next.js process or remove it from
   service.
2. Stop `proexel-service` and confirm no process has the state file open for
   writing.
3. Create a dated backup directory on a filesystem with enough free space.
4. Copy the canonical JSON file and attachment directory while preserving metadata.
5. Record a SHA-256 checksum and the deployed application/runtime revisions.
6. Validate that the copied file is valid JSON with `schema_version` equal to
   the expected release schema.
7. Restart the Rust service, verify health, then restore web traffic.

Example after both processes have stopped:

```bash
BACKUP_DIR="/var/backups/proexel/$(date -u +%Y%m%dT%H%M%SZ)"
STATE="proexel/apps/service/target/runtime/storage/proexel-state-v1.json"
mkdir -p "$BACKUP_DIR"
cp -p "$STATE" "$BACKUP_DIR/proexel-state-v1.json"
cp -Rp proexel/apps/service/target/runtime/attachments "$BACKUP_DIR/attachments"
shasum -a 256 "$BACKUP_DIR/proexel-state-v1.json" > "$BACKUP_DIR/SHA256SUMS"
jq -e '.schema_version == 1' "$BACKUP_DIR/proexel-state-v1.json"
```

`jq` is a validation convenience, not a runtime dependency. On Linux,
`sha256sum` may be used instead of `shasum -a 256`.

## Restore

Restoring replaces the complete canonical state, including idempotency receipts
and audit events. Never merge files manually.

1. Stop web traffic and `proexel-service`.
2. Back up the current state separately, even when it is suspected to be bad.
3. Verify the selected backup checksum.
4. Validate JSON and confirm its `schema_version` is supported by the deployed
   binary.
5. Restore attachments to a temporary sibling directory and copy the state to a
   temporary file in the canonical directory.
6. Set the expected owner and restrictive file mode.
7. Rename the temporary attachment directory and state file into place before
   either process starts.
8. Start only `proexel-service`; verify `/v1/health` and representative queries.
9. Start Next.js, sign in and compare overview/entity counts with the backup
   record before reopening traffic.

Example replacement after validation:

```bash
STATE="proexel/apps/service/target/runtime/storage/proexel-state-v1.json"
cp -p /var/backups/proexel/<backup>/proexel-state-v1.json "${STATE}.restore"
chmod 600 "${STATE}.restore"
mv "${STATE}.restore" "$STATE"
```

## Restore verification

- The service starts without `storage_decode_failed`.
- The health endpoint returns success.
- `schema_version` is the expected value.
- User, valve, maintenance, order, stock, supplier and audit counts are plausible.
- A read-only UI smoke test succeeds for an authorized role.
- A controlled write creates exactly one audit event and survives service
  restart.

## Recovery constraints

- A backup taken while the writer is active is not an approved recovery point,
  even if it happens to parse.
- A newer schema must not be loaded into an older binary.
- `processed_commands` must be retained; removing it can make old retries apply
  a business action twice.
- State and attachments must share one consistency point. Orphaned files are
  harmless but missing referenced files break photo/signature viewing.
- User accounts and credential hashes are canonical state. Encrypt backups and
  restrict them as credential material. `PROEXEL_AUTH_USERS` is only an initial
  seed and must not overwrite restored accounts.

## Backup test cadence

At least once per release, restore the latest backup into an isolated path via
`PROEXEL_DATA_FILE`, start the service on an isolated deployment/listener and
run the restore verification checklist. A backup is not considered valid until
a restore has been tested.
