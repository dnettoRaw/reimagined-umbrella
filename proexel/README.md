# PROEXEL rebuild workspace

This workspace contains the new PROEXEL application. It consumes AppCore as a
runtime boundary and keeps PROEXEL business rules outside `core/AppCore-Runtime`.

## Layout

- `apps/service`: AppCore-hosted application service.
- `apps/web`: Next.js user interface, authentication/session boundary, and
  server-side AppCore transport adapter.
- `crates/proexel-domain`: product aggregates, value objects and policies.
- `crates/proexel-application`: commands, queries, DTO boundaries and RBAC.
- `crates/proexel-infrastructure`: storage, audit, sync and external adapters.
- `crates/proexel-migration`: repeatable legacy import/migration tooling.

## Run the local stack

The development launcher rotates a three-hour local secret, creates one scoped
token per AppCore capability, starts the Rust service, and starts Next.js:

```bash
./proexel/scripts/dev-stack.sh
```

The launcher prints a one-time local administrator login. Open
`http://localhost:3000/auth/login`. `PROEXEL_AUTH_USERS` seeds accounts only when
canonical user state is empty; use **Users and access** afterward to create,
assign, disable and reset password/PIN credentials for the four roles. Local canonical state is
persisted at
`proexel/apps/service/target/runtime/storage/proexel-state-v1.json`.

The interface is fully localized in Portuguese, English, Spanish, and French.
Users can change language from the login page or authenticated header; the
preference is stored in the `proexel_locale` cookie.

## Checks

```bash
cargo test --manifest-path proexel/Cargo.toml
```

For the web application:

```bash
cd proexel/apps/web
npm run check
npx tsc --noEmit
npm run build
npm audit --audit-level=high
```

Run the isolated, ephemeral browser workflow from the repository root:

```bash
./proexel/scripts/e2e.sh
```

Photo bytes default to
`proexel/apps/service/target/runtime/attachments`; set
`PROEXEL_ATTACHMENTS_DIR` to an absolute private path in deployed environments.

## Local AppCore smoke test

Generate the local runtime secret before starting the service. AppCore writes
short-lived secret metadata for this simple file flow, so regenerate it when
starting a new local session.

```bash
mkdir -p proexel/apps/service/target/runtime
cargo run --manifest-path core/AppCore-Runtime/Cargo.toml -p appcore-bin --bin appcore-bin -- \
  security secret rotate \
  --deployment proexel/apps/service/deployment.local.toml \
  --out proexel/apps/service/target/runtime/runtime-security.secret
chmod 600 proexel/apps/service/target/runtime/runtime-security.secret

APPCORE_APPLICATION_MANIFEST=proexel/apps/service/application.toml \
APPCORE_DEPLOYMENT_MANIFEST=proexel/apps/service/deployment.local.toml \
cargo run --manifest-path proexel/Cargo.toml -p proexel-service
```

To call `proexel.machines.list` directly:

```bash
TOKEN=$(cargo run --quiet --manifest-path core/AppCore-Runtime/Cargo.toml -p appcore-bin --bin appcore-bin -- \
  token query \
  --deployment proexel/apps/service/deployment.local.toml \
  --query proexel.machines.list)

curl -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query_name":"proexel.machines.list","query_id":"qry-local-smoke","payload":{"actor":{"id":"smoke-admin","name":"Smoke Admin","role":"admin"},"data":{}}}' \
  http://127.0.0.1:39400/v1/query
```

## Legacy migration

Prepare a JSON bundle with the top-level collections documented in
[`../docs/migration-runbook.md`](../docs/migration-runbook.md), then dry-run it
before writing the canonical state:

```bash
cargo run --manifest-path proexel/Cargo.toml -p proexel-migration --bin proexel-migrate -- \
  --input export/legacy.json \
  --state proexel/apps/service/target/runtime/storage/proexel-state-v1.json \
  --batch legacy-2026-08-13 \
  --dry-run \
  --report-json target/migration-report.json \
  --report-markdown target/migration-report.md
```

Use `proexel/fixtures/legacy-example.json` as the input-contract reference.

## Documentation

The maintained documentation index is [`../docs/README.md`](../docs/README.md).
It links architecture, configuration, RBAC, deployment, backup/restore,
migration, troubleshooting, release status, and functional parity.
