# PROEXEL standalone deployment

This document covers the implemented standalone topology. Distributed AppCore
deployment and remote synchronization are not configured in the repository.

## Supported topology

- One `proexel-service` process as the only writer of the canonical state file.
- One or more Next.js instances only if they share the same session secret and
  user configuration. The current login rate limiter is process-local, so a
  multi-instance web deployment requires an external rate-limit adapter before
  production use.
- The Rust listener remains on loopback or a protected private interface.
- A TLS reverse proxy terminates public traffic in front of Next.js.
- No public route directly exposes AppCore command/query endpoints.

## Build artifacts

From the repository root:

```bash
cargo build --release --manifest-path core/AppCore-Runtime/Cargo.toml \
  -p appcore-bin --bin appcore-bin
cargo build --release --manifest-path proexel/Cargo.toml \
  -p proexel-service -p proexel-migration
cd proexel/apps/web
npm ci
npm run check
npm run build
```

Run `cargo test --workspace` from `proexel/` and `npx tsc --noEmit` from the web
directory as release gates. `cargo audit` and `npm audit --audit-level=high`
should also pass for the locked dependencies.

## Runtime account and directories

Use an unprivileged service account. Create storage, attachment, backup and secret
directories owned by that account. Recommended modes are `0700` for directories
and `0600` for the runtime secret, state file and environment file.

Do not run the service with a working directory where the relative manifest
paths resolve somewhere unexpected. Prefer absolute paths for production
environment values and deployment manifests.

## Startup preparation

1. Install the exact AppCore and PROEXEL release artifacts.
2. Install reviewed application/deployment manifests outside a writable web
   root.
3. Create or rotate the AppCore runtime security secret.
4. Generate scoped command/query tokens for every capability in
   `application.toml` and expose the JSON map only to Next.js.
5. Load `PROEXEL_SESSION_SECRET`; on first bootstrap only, provide an active
   administrator through `PROEXEL_AUTH_USERS` to the Rust service.
6. Set `PROEXEL_DATA_FILE` to an absolute state path when not using the local
   manifest layout.
7. Set `PROEXEL_ATTACHMENTS_DIR` to an absolute private path owned by Next.js.
8. Start the Rust service and wait for health before starting Next.js.

The development launcher automates these steps with three-hour tokens and an
ephemeral administrator. Do not use `scripts/dev-stack.sh` as the production
service manager.

## Process environment

Rust service minimum:

```text
APPCORE_APPLICATION_MANIFEST=/etc/proexel/application.toml
APPCORE_DEPLOYMENT_MANIFEST=/etc/proexel/deployment.toml
PROEXEL_DATA_FILE=/var/lib/proexel/proexel-state-v1.json
PROEXEL_AUTH_USERS=<first-bootstrap server-only JSON array>
```

Next.js minimum:

```text
NODE_ENV=production
PROEXEL_SERVICE_URL=http://127.0.0.1:39400
PROEXEL_SERVICE_TOKENS=<server-only JSON map>
PROEXEL_SESSION_SECRET=<server-only random secret>
PROEXEL_ATTACHMENTS_DIR=/var/lib/proexel/attachments
```

See [configuration.md](configuration.md) for the complete contract.

## Startup order and health

1. Start `proexel-service`.
2. Call `GET http://127.0.0.1:39400/v1/health` until it is ready.
3. Exercise one scoped overview query from the host.
4. Start Next.js with `npm start` from the built web application.
5. Check `/auth/login`, authenticate with a non-admin smoke-test account and
   verify the role-appropriate overview/navigation.

The deployment is ready only when both process health and an authenticated data
query succeed. A rendered login page alone is insufficient.

## Reverse proxy requirements

- Serve only HTTPS externally.
- Forward the original client address through a trusted, sanitized header.
- Do not expose the Rust listener publicly.
- Preserve `HttpOnly`, `Secure` and `SameSite` cookie attributes. The application
  sets `Secure` automatically when `NODE_ENV=production`.
- Apply request/body limits appropriate for JSON operations.
- Keep the application attachment limits in place and add a proxy body limit
  slightly above 5 MB.

## Upgrade

1. Confirm a tested backup and record current binary/manifests.
2. Drain web traffic and stop both processes.
3. Build/install the new artifacts without deleting the old release.
4. Review schema and migration notes. The current schema is version 1.
5. Start the Rust service and verify health/query behavior.
6. Start Next.js and run role, locale and representative write smoke tests.
7. Observe logs and audit output before completing the rollout.

## Rollback

Application rollback and data rollback are separate decisions. If the state
schema is still compatible, stop both processes and restore the previous
artifacts/manifests. If state must also be rolled back, follow
[backup-restore.md](backup-restore.md). Never start an older binary against an
unsupported newer state schema.

## Production blockers

Before declaring this baseline production-ready, resolve the open items in
[functional-parity-checklist.md](functional-parity-checklist.md), especially
durable distributed rate limiting if scaling the web tier, cutover evidence and
explicit sync requirements for any future distributed topology.
