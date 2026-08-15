# PROEXEL migration runbook

## Input contract

The migration CLI accepts one JSON object with optional arrays named `valves`,
`maintenance_records`, `orders`, `restock_requests`, `stock`, `suppliers`, and
`valve_photos`. Portuguese legacy aliases such as `zona`, `marca`, `kit`,
`observacoes`, `valveTag`, `minQuantity`, and `kitChanged` are accepted.

Passwords and user credentials are intentionally outside this import contract.
Identity migration requires a separate authenticated process.

`proexel/fixtures/legacy-example.json` is an executable example covering all
supported top-level collections and the main Portuguese aliases.

## Procedure

1. Record a batch ID, source-system identifier, export timestamp, and operator.
2. Stop writes to the legacy system and export all source collections.
3. Validate that the export is one JSON object and retain its original checksum.
4. Back up the canonical state as described in
   [`backup-restore.md`](backup-restore.md).
5. Run `proexel-migrate` with `--dry-run` and both report outputs.
6. Review skipped records, unresolved valve links, brand/location warnings, and
   source-versus-imported counts.
7. Run the same command without `--dry-run` and with the same batch ID.
8. Run it a second time. Imported counts must be zero.
9. Start the stack and compare valve, maintenance, stock, order, supplier, and
   photo metadata counts against the report.

The release smoke test executes these three phases against a temporary state:

```bash
cargo run --manifest-path proexel/Cargo.toml -p proexel-migration --bin proexel-migrate -- \
  --input proexel/fixtures/legacy-example.json \
  --state /tmp/proexel-migration-state.json \
  --batch smoke-legacy-001 --dry-run \
  --report-json /tmp/proexel-migration-report.json \
  --report-markdown /tmp/proexel-migration-report.md
```

Repeat without `--dry-run`, then repeat once more. The example imports one of
each entity on the first write and zero records on the second write.

The tool generates stable IDs from normalized business identities, records a
batch audit event, and computes an FNV-1a checksum over the source bundle. It
does not debit stock while importing historical maintenance because the legacy
stock export already represents its current balance.

## Acceptance checks

- Dry-run leaves the state-file checksum unchanged.
- The first write reports the reviewed import and warning counts.
- Repeating the same batch imports zero records.
- Every imported maintenance and photo metadata record resolves to an immutable
  valve ID.
- Stock quantities remain non-negative and match the legacy cutover snapshot.
- The audit view contains the migration batch event and source checksum.
- Users and credentials are provisioned separately; they are never imported by
  this CLI.
- Photo binaries are retained in the source archive until a supported upload
  provider and attachment migration are implemented.

## Rollback

Stop the service and follow the restore procedure in
[`backup-restore.md`](backup-restore.md) using the pre-import backup. Verify file
ownership and health before reopening writes. Do not edit a partially migrated
state file manually.
