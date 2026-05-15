# Database Backup And Restore Contribution Scope

## Goal

Move project database backup and restore from a manual path-only flow to a managed workflow with safe artifacts, retention, compression, encryption, and scheduled execution while preserving the Rust backend as the security boundary.

## Current Implementation

- MySQL and PostgreSQL backups are still created through their official CLI dump tools, but execution stays behind the existing allowlisted database CLI boundary.
- Backup artifacts are finalized into app-managed files with metadata sidecars.
- Managed backups support gzip compression and AES-256-GCM encryption.
- Backup encryption keys are stored through the secure storage abstraction instead of frontend state or plain text config.
- Restore supports plain SQL and managed compressed or encrypted backup artifacts.
- Retention pruning removes expired managed backup artifacts after successful backup finalization.
- Backup policies are persisted per project and database type.
- Scheduled backup checks run through an application use case and can execute due policies while the desktop app is open.
- The Databases screen exposes backup options, restore file picker, schedule controls, and a manual due-backup sweep.

## Safety Rules

- Tauri command handlers must call application use cases only.
- Database dumps and restores must continue to use allowlisted infrastructure adapters.
- Backup encryption keys must never cross into frontend state.
- Restore paths must be selected through desktop file picker UX or validated backend input.
- Scheduled backups must stay encrypted by default.
- Retention pruning must only remove artifacts inside the project-owned backup directory.

## Still Out Of Scope

- OS-level background scheduler integration when the desktop app is closed.
- Remote backup destinations.
- Point-in-time recovery.
- Database-specific migration rollback orchestration.
- Backup artifact signing or external key management.
