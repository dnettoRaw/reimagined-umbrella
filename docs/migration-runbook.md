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

1. Stop writes to the legacy system and export all source collections.
2. Back up the canonical state as described in `docs/operations.md`.
3. Run `proexel-migrate` with `--dry-run` and both report outputs.
4. Review skipped records, unresolved valve links, brand/location warnings, and
   source-versus-imported counts.
5. Run the same command without `--dry-run` and with the same batch ID.
6. Run it a second time. Imported counts must be zero.
7. Start the stack and compare valve, maintenance, stock, order, supplier, and
   photo counts against the report.

The release smoke test executes these three phases against a temporary state:

```bash
cargo run --manifest-path proexel/Cargo.toml -p proexel-migration --bin proexel-migrate -- \
  --input proexel/fixtures/legacy-example.json \
  --state /tmp/proexel-migration-state.json \
  --batch smoke-legacy-001 --dry-run
```

Repeat without `--dry-run`, then repeat once more. The example imports one of
each entity on the first write and zero records on the second write.

The tool generates stable IDs from normalized business identities, records a
batch audit event, and computes an FNV-1a checksum over the source bundle. It
does not debit stock while importing historical maintenance because the legacy
stock export already represents its current balance.

## Rollback

Stop the service, replace `proexel-state-v1.json` with the pre-import backup,
verify file ownership, then restart the stack. Do not edit a partially migrated
state file manually.
