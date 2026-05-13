use tauri::State;

use crate::application::databases::backup_project_database_use_case;
use crate::application::databases::create_project_database_migration_use_case;
use crate::application::databases::list_project_database_profiles_use_case;
use crate::application::databases::provision_project_database_use_case;
use crate::application::databases::restore_project_database_use_case;
use crate::application::databases::run_project_database_migrations_use_case;
use crate::bootstrap::app_state::AppState;
use crate::domain::database::database_config::{
    DatabaseBackupResult, DatabaseMigrationFile, DatabaseMigrationRunResult,
    DatabaseProvisioningResult, DatabaseRestoreResult, ProjectDatabaseProfile,
};
use crate::shared::error::command_error_mapper::{map_command_error, CommandErrorPayload};

#[tauri::command]
pub fn list_project_database_profiles(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<ProjectDatabaseProfile>, CommandErrorPayload> {
    list_project_database_profiles_use_case::list_project_database_profiles(
        state.project_repository(),
        state.database_provisioning_repository(),
        &project_id,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database profile list command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn provision_project_database(
    state: State<'_, AppState>,
    project_id: String,
    database_type: String,
) -> Result<DatabaseProvisioningResult, CommandErrorPayload> {
    provision_project_database_use_case::provision_project_database(
        state.project_repository(),
        state.database_provisioning_repository(),
        state.database_provisioner(),
        &project_id,
        &database_type,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database provisioning command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn backup_project_database(
    state: State<'_, AppState>,
    project_id: String,
    database_type: String,
) -> Result<DatabaseBackupResult, CommandErrorPayload> {
    backup_project_database_use_case::backup_project_database(
        state.database_provisioning_repository(),
        state.database_provisioner(),
        &project_id,
        &database_type,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database backup command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn restore_project_database(
    state: State<'_, AppState>,
    project_id: String,
    database_type: String,
    backup_path: String,
) -> Result<DatabaseRestoreResult, CommandErrorPayload> {
    restore_project_database_use_case::restore_project_database(
        state.database_provisioning_repository(),
        state.database_provisioner(),
        &project_id,
        &database_type,
        &backup_path,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database restore command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn create_project_database_migration(
    state: State<'_, AppState>,
    project_id: String,
    database_type: String,
    name: String,
) -> Result<DatabaseMigrationFile, CommandErrorPayload> {
    create_project_database_migration_use_case::create_project_database_migration(
        state.database_provisioning_repository(),
        state.database_provisioner(),
        &project_id,
        &database_type,
        &name,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database migration file command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn run_project_database_migrations(
    state: State<'_, AppState>,
    project_id: String,
    database_type: String,
) -> Result<DatabaseMigrationRunResult, CommandErrorPayload> {
    run_project_database_migrations_use_case::run_project_database_migrations(
        state.database_provisioning_repository(),
        state.database_provisioner(),
        &project_id,
        &database_type,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database migration run command failed");
        map_command_error(&error)
    })
}
