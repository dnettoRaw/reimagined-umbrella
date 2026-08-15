# PROEXEL web application

Next.js 16 App Router interface for the PROEXEL industrial asset and guided-maintenance system. This
application owns the browser session, page-level access checks, localized UI,
and server-side proxy to the AppCore-hosted PROEXEL service. It does not read or
write canonical state directly.

## Run locally

From the repository root, use the stack launcher so that AppCore capability
tokens and the development login are generated together:

```bash
./proexel/scripts/dev-stack.sh
```

Open `http://localhost:3000/auth/login` and use the one-run administrator
credential printed by the launcher.

To run only Next.js, provide the server-only variables described in
[`../../../docs/configuration.md`](../../../docs/configuration.md), then run:

```bash
cd proexel/apps/web
npm install
npm run dev
```

## Responsibilities

- Authenticate canonical active users with server-side scrypt password/PIN verification.
- Manage users, roles, status and credential resets through audited admin capabilities.
- Store signed `HttpOnly`, `SameSite=Strict` sessions.
- Enforce route and command access using the application RBAC matrix.
- Call AppCore through scoped command/query tokens held only on the server.
- Render machines, component categories, structured guides, service orders,
  guided inspections, purchasing, stock, reports, audit, operators, and settings.
- Render all application-owned text in Portuguese, English, Spanish, or French.
- Validate and serve authenticated machine, component, guide, inspection, and
  replacement photos from the protected attachment adapter.

Translations live under `src/lib/i18n`. New visible text must be added to all
four locale dictionaries in the same change; Portuguese is the fallback locale.

## Checks

```bash
cd proexel/apps/web
npm run check
npx tsc --noEmit
npm run build
npm audit --audit-level=high
```

The committed browser workflow is run from the repository root with
`./proexel/scripts/e2e.sh`. It uses isolated ports, state, attachments, build
output and ephemeral credentials.

The application requires a running service for meaningful page and command
tests. The full local topology is documented in
[`../../../docs/deployment.md`](../../../docs/deployment.md), and current
feature gaps are tracked in
[`../../../docs/functional-parity-checklist.md`](../../../docs/functional-parity-checklist.md).
