import { invokeTauriCommand } from "../../../core/api/tauri-client";
import type {
  DatabaseBackupResult,
  DatabaseMigrationFile,
  DatabaseMigrationRunResult,
  DatabaseProvisioningResult,
  DatabaseRestoreResult,
  DatabaseType,
  ProjectDatabaseProfile,
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

export function backupProjectDatabase(projectId: string, databaseType: DatabaseType) {
  return invokeTauriCommand<DatabaseBackupResult>("backup_project_database", {
    projectId,
    databaseType,
  });
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

export function runProjectDatabaseMigrations(projectId: string, databaseType: DatabaseType) {
  return invokeTauriCommand<DatabaseMigrationRunResult>("run_project_database_migrations", {
    projectId,
    databaseType,
  });
}
