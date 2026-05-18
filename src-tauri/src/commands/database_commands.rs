use tauri::State;

use crate::application::databases::backup_project_database_use_case;
use crate::application::databases::create_project_database_migration_use_case;
use crate::application::databases::export_database_backup_trust_bundle_use_case;
use crate::application::databases::generate_project_database_migration_rollback_use_case;
use crate::application::databases::get_database_backup_key_management_status_use_case;
use crate::application::databases::get_database_backup_scheduler_status_use_case;
use crate::application::databases::import_database_backup_trust_bundle_use_case;
use crate::application::databases::install_database_backup_scheduler_use_case;
use crate::application::databases::list_database_backup_destinations_use_case;
use crate::application::databases::list_database_backup_policies_use_case;
use crate::application::databases::list_project_database_profiles_use_case;
use crate::application::databases::provision_project_database_use_case;
use crate::application::databases::restore_project_database_to_point_in_time_use_case;
use crate::application::databases::restore_project_database_use_case;
use crate::application::databases::restore_project_database_with_replay_use_case;
use crate::application::databases::rollback_project_database_migrations_use_case;
use crate::application::databases::run_due_database_backups_use_case;
use crate::application::databases::run_project_database_migrations_use_case;
use crate::application::databases::uninstall_database_backup_scheduler_use_case;
use crate::application::databases::update_database_backup_destination_use_case;
use crate::application::databases::update_database_backup_policy_use_case;
use crate::bootstrap::app_state::AppState;
use crate::domain::database::database_config::{
    DatabaseBackupKeyManagementStatus, DatabaseBackupOptions, DatabaseBackupPolicy,
    DatabaseBackupPolicyUpdate, DatabaseBackupPolicyUpdateResult, DatabaseBackupRemoteDestination,
    DatabaseBackupRemoteDestinationUpdate, DatabaseBackupRemoteDestinationUpdateResult,
    DatabaseBackupResult, DatabaseBackupSchedulerInstallResult, DatabaseBackupSchedulerStatus,
    DatabaseBackupTrustExportResult, DatabaseBackupTrustImportResult,
    DatabaseContinuousReplayRestoreResult, DatabaseMigrationFile,
    DatabaseMigrationRollbackGenerationResult, DatabaseMigrationRollbackResult,
    DatabaseMigrationRunResult, DatabasePointInTimeRestoreResult, DatabaseProvisioningResult,
    DatabaseRestoreResult, ProjectDatabaseProfile, ScheduledDatabaseBackupRunResult,
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
        state.database_dependency_manager(),
        state.database_provisioner(),
        state.service_manager(),
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
    options: Option<DatabaseBackupOptions>,
) -> Result<DatabaseBackupResult, CommandErrorPayload> {
    backup_project_database_use_case::backup_project_database(
        state.database_provisioning_repository(),
        state.database_backup_destination_repository(),
        state.database_provisioner(),
        &project_id,
        &database_type,
        options,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database backup command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn list_database_backup_destinations(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<DatabaseBackupRemoteDestination>, CommandErrorPayload> {
    list_database_backup_destinations_use_case::list_database_backup_destinations(
        state.database_backup_destination_repository(),
        &project_id,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database backup destination list command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn update_database_backup_destination(
    state: State<'_, AppState>,
    project_id: String,
    database_type: String,
    update: DatabaseBackupRemoteDestinationUpdate,
) -> Result<DatabaseBackupRemoteDestinationUpdateResult, CommandErrorPayload> {
    update_database_backup_destination_use_case::update_database_backup_destination(
        state.database_backup_destination_repository(),
        state.database_provisioning_repository(),
        &project_id,
        &database_type,
        update,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database backup destination update command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn list_database_backup_policies(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Vec<DatabaseBackupPolicy>, CommandErrorPayload> {
    list_database_backup_policies_use_case::list_database_backup_policies(
        state.database_backup_policy_repository(),
        &project_id,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database backup policy list command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn update_database_backup_policy(
    state: State<'_, AppState>,
    project_id: String,
    database_type: String,
    update: DatabaseBackupPolicyUpdate,
) -> Result<DatabaseBackupPolicyUpdateResult, CommandErrorPayload> {
    update_database_backup_policy_use_case::update_database_backup_policy(
        state.database_backup_policy_repository(),
        state.database_provisioning_repository(),
        &project_id,
        &database_type,
        update,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database backup policy update command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn run_due_database_backups(
    state: State<'_, AppState>,
) -> Result<ScheduledDatabaseBackupRunResult, CommandErrorPayload> {
    run_due_database_backups_use_case::run_due_database_backups(
        state.database_backup_policy_repository(),
        state.database_backup_destination_repository(),
        state.database_provisioning_repository(),
        state.database_provisioner(),
    )
    .map_err(|error| {
        tracing::warn!(?error, "scheduled database backup command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn get_database_backup_scheduler_status(
    state: State<'_, AppState>,
) -> Result<DatabaseBackupSchedulerStatus, CommandErrorPayload> {
    get_database_backup_scheduler_status_use_case::get_database_backup_scheduler_status(
        state.database_backup_scheduler(),
    )
    .map_err(|error| {
        tracing::warn!(?error, "database backup scheduler status command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn get_database_backup_key_management_status(
    state: State<'_, AppState>,
) -> Result<DatabaseBackupKeyManagementStatus, CommandErrorPayload> {
    get_database_backup_key_management_status_use_case::get_database_backup_key_management_status(
        state.secure_storage(),
    )
    .map_err(|error| {
        tracing::warn!(
            ?error,
            "database backup key management status command failed"
        );
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn export_database_backup_trust_bundle(
    state: State<'_, AppState>,
    output_dir: String,
) -> Result<DatabaseBackupTrustExportResult, CommandErrorPayload> {
    export_database_backup_trust_bundle_use_case::export_database_backup_trust_bundle(
        state.secure_storage(),
        &output_dir,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database backup trust export command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn import_database_backup_trust_bundle(
    state: State<'_, AppState>,
    trust_bundle_path: String,
) -> Result<DatabaseBackupTrustImportResult, CommandErrorPayload> {
    import_database_backup_trust_bundle_use_case::import_database_backup_trust_bundle(
        state.secure_storage(),
        &trust_bundle_path,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database backup trust import command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn install_database_backup_scheduler(
    state: State<'_, AppState>,
) -> Result<DatabaseBackupSchedulerInstallResult, CommandErrorPayload> {
    install_database_backup_scheduler_use_case::install_database_backup_scheduler(
        state.database_backup_scheduler(),
    )
    .map_err(|error| {
        tracing::warn!(?error, "database backup scheduler install command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn uninstall_database_backup_scheduler(
    state: State<'_, AppState>,
) -> Result<DatabaseBackupSchedulerInstallResult, CommandErrorPayload> {
    uninstall_database_backup_scheduler_use_case::uninstall_database_backup_scheduler(
        state.database_backup_scheduler(),
    )
    .map_err(|error| {
        tracing::warn!(?error, "database backup scheduler uninstall command failed");
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
pub fn restore_project_database_to_point_in_time(
    state: State<'_, AppState>,
    project_id: String,
    database_type: String,
    target_time: String,
) -> Result<DatabasePointInTimeRestoreResult, CommandErrorPayload> {
    restore_project_database_to_point_in_time_use_case::restore_project_database_to_point_in_time(
        state.database_provisioning_repository(),
        state.database_backup_catalog(),
        state.database_provisioner(),
        &project_id,
        &database_type,
        &target_time,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database point-in-time restore command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn restore_project_database_with_replay(
    state: State<'_, AppState>,
    project_id: String,
    database_type: String,
    base_backup_path: String,
    replay_source_path: String,
    target_time: Option<String>,
) -> Result<DatabaseContinuousReplayRestoreResult, CommandErrorPayload> {
    restore_project_database_with_replay_use_case::restore_project_database_with_replay(
        state.database_provisioning_repository(),
        state.database_provisioner(),
        &project_id,
        &database_type,
        &base_backup_path,
        &replay_source_path,
        target_time,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database continuous replay restore command failed");
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
pub fn generate_project_database_migration_rollback(
    state: State<'_, AppState>,
    project_id: String,
    database_type: String,
    migration_path: String,
) -> Result<DatabaseMigrationRollbackGenerationResult, CommandErrorPayload> {
    generate_project_database_migration_rollback_use_case::generate_project_database_migration_rollback(
        state.database_provisioning_repository(),
        state.database_provisioner(),
        &project_id,
        &database_type,
        &migration_path,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database migration rollback generation command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn rollback_project_database_migrations(
    state: State<'_, AppState>,
    project_id: String,
    database_type: String,
    steps: u16,
) -> Result<DatabaseMigrationRollbackResult, CommandErrorPayload> {
    rollback_project_database_migrations_use_case::rollback_project_database_migrations(
        state.database_provisioning_repository(),
        state.database_provisioner(),
        &project_id,
        &database_type,
        steps,
    )
    .map_err(|error| {
        tracing::warn!(?error, "database migration rollback command failed");
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
