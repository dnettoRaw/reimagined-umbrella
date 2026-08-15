# PROEXEL documentation - CONCLUÍDO 100%

This directory describes the rebuilt PROEXEL application. Paths and commands
assume the repository root as the current directory unless stated otherwise.

## Start here

- [Architecture](architecture.md): component boundaries, request flow,
  persistence and trust boundaries.
- [Configuration](configuration.md): manifests, environment variables, users,
  tokens and local paths.
- [Deployment](deployment.md): standalone build, startup, health checks and
  rollback expectations.
- [Operations](operations.md): runtime ownership and daily operational model.
- [Backup and restore](backup-restore.md): consistent cold backups and tested
  restore procedure.
- [Troubleshooting](troubleshooting.md): symptom-based diagnostics.
- [Migration runbook](migration-runbook.md): deterministic import of legacy
  datasets.

## Product and design records

- [Legacy behavior map](legacy-behavior-map.md)
- [Legacy data map](legacy-data-map.md)
- [Functional parity checklist](functional-parity-checklist.md)
- [RBAC matrix](rbac-matrix.md)
- [AppCore integration surface](appcore-integration-surface.md)
- [AppCore consumer boundary ADR](adr/0001-appcore-consumer-boundary.md)
- [Implementation status](implementation-status.md)
- [Release notes](release-notes.md)

## Source contracts

- AppCore application manifest:
  `proexel/apps/service/application.toml`
- Local deployment manifest:
  `proexel/apps/service/deployment.local.toml`
- Environment template: `proexel/.env.example`
- Domain schema and models: `proexel/crates/proexel-domain/src/model.rs`
- Command and query names:
  `proexel/crates/proexel-application/src/commands.rs`
- Canonical state schema:
  `proexel/crates/proexel-application/src/state.rs`

The `PROEXEL_REBUILD_PROMPTS/` directory records the completed delivery phases.
This directory is the operational documentation for the resulting application.
