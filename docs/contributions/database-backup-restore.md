# Database Backup And Restore Contribution Scope

## Goal

Move project database backup and restore from a manual path-only flow to a managed workflow with safe artifacts, retention, compression, encryption, scheduled execution, point-in-time restore, rollback orchestration, and externalized trust boundaries while preserving the Rust backend as the security boundary.

## Current Implementation

- MySQL and PostgreSQL backups are still created through their official CLI dump tools, but execution stays behind the existing allowlisted database CLI boundary.
- Backup artifacts are finalized into app-managed files with metadata sidecars.
- Managed backups support gzip compression and AES-256-GCM encryption.
- Backup encryption and signing keys are stored through the secure storage abstraction instead of frontend state or plain text config, with optional external key injection through environment variables for managed deployments.
- Backup artifacts are signed with HMAC-SHA256 sidecars before restore verification.
- Remote backup destinations can copy managed backup, metadata, and signature artifacts to an absolute mounted destination path.
- Restore supports plain SQL and managed compressed or encrypted backup artifacts.
- Point-in-time restore selects the newest managed backup metadata at or before a requested RFC3339 target timestamp, then restores through the same provisioner boundary.
- Retention pruning removes expired managed backup artifacts after successful backup finalization.
- Backup policies are persisted per project and database type.
- Scheduled backup checks run through an application use case and can execute due policies while the desktop app is open or through the OS scheduler CLI entrypoint.
- macOS LaunchAgent and Windows Task Scheduler adapters are prepared behind a scheduler port so background backup sweeps can run after the desktop app is closed.
- Migration rollback creates paired `.down.sql` files and executes rollback steps through the same allowlisted database CLI boundary.
- The Databases screen exposes backup options, restore file picker, remote destination controls, schedule controls, OS scheduler controls, point-in-time restore, migration rollback, and a manual due-backup sweep.

## Safety Rules

- Tauri command handlers must call application use cases only.
- Database dumps and restores must continue to use allowlisted infrastructure adapters.
- Backup encryption keys must never cross into frontend state.
- Backup signing keys must never cross into frontend state.
- Restore paths must be selected through desktop file picker UX or validated backend input.
- Scheduled backups must stay encrypted by default.
- Retention pruning must only remove artifacts inside the project-owned backup directory.
- OS scheduler adapters must invoke only the app CLI entrypoint, not arbitrary shell commands.
- Remote backup destinations must be absolute directories and must not receive credentials.
- Point-in-time restore must select from managed metadata only.
- Migration rollback must require explicit rollback files and must update applied migration state only after successful execution.

## Still Out Of Scope

- Cloud-native remote backup providers such as S3, R2, GCS, or SFTP.
- Continuous WAL/binlog point-in-time recovery beyond managed snapshot selection.
- Automatic rollback generation from forward migration SQL.
- External KMS integrations beyond environment-provided key material.
- Cross-machine backup artifact restore trust workflows.
