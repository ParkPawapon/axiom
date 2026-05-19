import { invokeTauriCommand } from "../../../core/api/tauri-client";
import type {
  DatabaseBackupOptions,
  DatabaseBackupArtifactTrustEnrollmentResult,
  DatabaseBackupKeyManagementStatus,
  DatabaseBackupPolicy,
  DatabaseBackupPolicyUpdate,
  DatabaseBackupPolicyUpdateResult,
  DatabaseBackupRemoteDestination,
  DatabaseBackupRemoteDestinationUpdate,
  DatabaseBackupRemoteDestinationUpdateResult,
  DatabaseBackupResult,
  DatabaseBackupSchedulerInstallResult,
  DatabaseBackupSchedulerStatus,
  DatabaseBackupTrustExportResult,
  DatabaseBackupTrustImportResult,
  DatabaseContinuousReplayRestoreResult,
  DatabaseMigrationFile,
  DatabaseMigrationRollbackGenerationResult,
  DatabaseMigrationRollbackResult,
  DatabaseMigrationRunResult,
  DatabasePointInTimeRestoreResult,
  DatabaseProvisioningResult,
  DatabaseRestoreResult,
  DatabaseType,
  ProjectDatabaseProfile,
  ScheduledDatabaseBackupRunResult,
} from "../types/database.types";

export function listProjectDatabaseProfiles(projectId: string) {
  return invokeTauriCommand<ProjectDatabaseProfile[]>("list_project_database_profiles", {
    projectId,
  });
}

export function provisionProjectDatabase(projectId: string, databaseType: DatabaseType) {
  return invokeTauriCommand<DatabaseProvisioningResult>("provision_project_database", {
    projectId,
    databaseType,
  });
}

export function backupProjectDatabase(
  projectId: string,
  databaseType: DatabaseType,
  options?: DatabaseBackupOptions,
) {
  return invokeTauriCommand<DatabaseBackupResult>("backup_project_database", {
    projectId,
    databaseType,
    options,
  });
}

export function listDatabaseBackupPolicies(projectId: string) {
  return invokeTauriCommand<DatabaseBackupPolicy[]>("list_database_backup_policies", {
    projectId,
  });
}

export function listDatabaseBackupDestinations(projectId: string) {
  return invokeTauriCommand<DatabaseBackupRemoteDestination[]>(
    "list_database_backup_destinations",
    {
      projectId,
    },
  );
}

export function updateDatabaseBackupPolicy(
  projectId: string,
  databaseType: DatabaseType,
  update: DatabaseBackupPolicyUpdate,
) {
  return invokeTauriCommand<DatabaseBackupPolicyUpdateResult>("update_database_backup_policy", {
    databaseType,
    projectId,
    update,
  });
}

export function updateDatabaseBackupDestination(
  projectId: string,
  databaseType: DatabaseType,
  update: DatabaseBackupRemoteDestinationUpdate,
) {
  return invokeTauriCommand<DatabaseBackupRemoteDestinationUpdateResult>(
    "update_database_backup_destination",
    {
      databaseType,
      projectId,
      update,
    },
  );
}

export function runDueDatabaseBackups() {
  return invokeTauriCommand<ScheduledDatabaseBackupRunResult>("run_due_database_backups");
}

export function getDatabaseBackupSchedulerStatus() {
  return invokeTauriCommand<DatabaseBackupSchedulerStatus>("get_database_backup_scheduler_status");
}

export function getDatabaseBackupKeyManagementStatus() {
  return invokeTauriCommand<DatabaseBackupKeyManagementStatus>(
    "get_database_backup_key_management_status",
  );
}

export function exportDatabaseBackupTrustBundle(outputDir: string) {
  return invokeTauriCommand<DatabaseBackupTrustExportResult>(
    "export_database_backup_trust_bundle",
    { outputDir },
  );
}

export function importDatabaseBackupTrustBundle(trustBundlePath: string) {
  return invokeTauriCommand<DatabaseBackupTrustImportResult>(
    "import_database_backup_trust_bundle",
    { trustBundlePath },
  );
}

export function enrollDatabaseBackupArtifactTrust(backupPath: string) {
  return invokeTauriCommand<DatabaseBackupArtifactTrustEnrollmentResult>(
    "enroll_database_backup_artifact_trust",
    { backupPath },
  );
}

export function installDatabaseBackupScheduler() {
  return invokeTauriCommand<DatabaseBackupSchedulerInstallResult>(
    "install_database_backup_scheduler",
  );
}

export function uninstallDatabaseBackupScheduler() {
  return invokeTauriCommand<DatabaseBackupSchedulerInstallResult>(
    "uninstall_database_backup_scheduler",
  );
}

export function restoreProjectDatabase(
  projectId: string,
  databaseType: DatabaseType,
  backupPath: string,
) {
  return invokeTauriCommand<DatabaseRestoreResult>("restore_project_database", {
    projectId,
    databaseType,
    backupPath,
  });
}

export function restoreProjectDatabaseToPointInTime(
  projectId: string,
  databaseType: DatabaseType,
  targetTime: string,
) {
  return invokeTauriCommand<DatabasePointInTimeRestoreResult>(
    "restore_project_database_to_point_in_time",
    {
      databaseType,
      projectId,
      targetTime,
    },
  );
}

export function restoreProjectDatabaseWithReplay(
  projectId: string,
  databaseType: DatabaseType,
  baseBackupPath: string,
  replaySourcePath: string,
  targetTime?: string,
) {
  return invokeTauriCommand<DatabaseContinuousReplayRestoreResult>(
    "restore_project_database_with_replay",
    {
      baseBackupPath,
      databaseType,
      projectId,
      replaySourcePath,
      targetTime,
    },
  );
}

export function createProjectDatabaseMigration(
  projectId: string,
  databaseType: DatabaseType,
  name: string,
) {
  return invokeTauriCommand<DatabaseMigrationFile>("create_project_database_migration", {
    projectId,
    databaseType,
    name,
  });
}

export function rollbackProjectDatabaseMigrations(
  projectId: string,
  databaseType: DatabaseType,
  steps: number,
) {
  return invokeTauriCommand<DatabaseMigrationRollbackResult>(
    "rollback_project_database_migrations",
    {
      databaseType,
      projectId,
      steps,
    },
  );
}

export function generateProjectDatabaseMigrationRollback(
  projectId: string,
  databaseType: DatabaseType,
  migrationPath: string,
) {
  return invokeTauriCommand<DatabaseMigrationRollbackGenerationResult>(
    "generate_project_database_migration_rollback",
    {
      databaseType,
      migrationPath,
      projectId,
    },
  );
}

export function runProjectDatabaseMigrations(projectId: string, databaseType: DatabaseType) {
  return invokeTauriCommand<DatabaseMigrationRunResult>("run_project_database_migrations", {
    projectId,
    databaseType,
  });
}
