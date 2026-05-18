# Database Backup And Restore Contribution Scope

## Goal

Move project database backup and restore from a manual path-only flow to a managed workflow with safe artifacts, retention, compression, encryption, scheduled execution, point-in-time restore, rollback orchestration, and externalized trust boundaries while preserving the Rust backend as the security boundary.

## Current Implementation

- MySQL and PostgreSQL backups are still created through their official CLI dump tools, but execution stays behind the existing allowlisted database CLI boundary.
- Backup artifacts are finalized into app-managed files with metadata sidecars.
- Managed backups support gzip compression and AES-256-GCM encryption.
- Backup encryption and signing keys are stored through the secure storage abstraction instead of frontend state or plain text config, with optional external key injection through environment variables for managed deployments.
- Backup artifacts are signed with HMAC-SHA256 sidecars before restore verification.
- Remote backup destinations can copy managed backup, metadata, and signature artifacts to an absolute mounted path, S3 URI, R2 URI through an `AXIOM_R2_ENDPOINT_URL`, GCS URI, or SFTP URI through provider-specific CLI adapters.
- Restore supports plain SQL and managed compressed or encrypted backup artifacts.
- Point-in-time restore selects the newest managed backup metadata at or before a requested RFC3339 target timestamp, then restores through the same provisioner boundary.
- Continuous replay restore can restore a base managed artifact and then replay sorted SQL recovery segments, plus MySQL binlog files converted through `mysqlbinlog`.
- Retention pruning removes expired managed backup artifacts after successful backup finalization.
- Backup policies are persisted per project and database type.
- Scheduled backup checks run through an application use case and can execute due policies while the desktop app is open or through the OS scheduler CLI entrypoint.
- macOS LaunchAgent and Windows Task Scheduler adapters are prepared behind a scheduler port so background backup sweeps can run after the desktop app is closed.
- Migration rollback creates paired `.down.sql` files, can generate conservative rollback SQL from reversible forward migration patterns, and executes rollback steps through the same allowlisted database CLI boundary.
- Backup trust enrollment can export/import a signing-key fingerprint bundle without exposing secret key material.
- Backup key-management status reports secure-storage, environment, and external KMS metadata without serializing key material to the frontend.
- The Databases screen exposes backup options, restore file picker, remote destination controls, schedule controls, OS scheduler controls, point-in-time restore, replay restore, migration rollback generation, trust bundle import/export, and a manual due-backup sweep.

## Safety Rules

- Tauri command handlers must call application use cases only.
- Database dumps and restores must continue to use allowlisted infrastructure adapters.
- Backup encryption keys must never cross into frontend state.
- Backup signing keys must never cross into frontend state.
- Restore paths must be selected through desktop file picker UX or validated backend input.
- Scheduled backups must stay encrypted by default.
- Retention pruning must only remove artifacts inside the project-owned backup directory.
- OS scheduler adapters must invoke only the app CLI entrypoint, not arbitrary shell commands.
- Remote backup destinations must be typed by provider and must not receive credentials through frontend state.
- Point-in-time restore must select from managed metadata only.
- Replay restore must apply only backend-validated files from a selected replay directory.
- Migration rollback must require explicit rollback files and must update applied migration state only after successful execution.
- Generated rollback SQL must be treated as reviewable SQL, not blindly trusted automation.
- Trust bundles must contain fingerprints only, never backup signing or encryption key material.

## Still Out Of Scope

- Native cloud SDK uploads without provider CLIs.
- PostgreSQL physical WAL server restore orchestration; current managed replay expects SQL replay segments for PostgreSQL.
- Semantic rollback generation for complex or destructive SQL beyond conservative reversible patterns.
- Direct KMS API envelope encryption; current external key management uses environment-provided key material plus provider/key-id metadata.
- Cross-machine decryption without shared external key material.
