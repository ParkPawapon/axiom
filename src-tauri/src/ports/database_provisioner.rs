use crate::domain::database::database_config::{
    DatabaseBackupOptions, DatabaseBackupResult, DatabaseContinuousReplayRestoreResult,
    DatabaseMigrationFile, DatabaseMigrationRollbackGenerationResult,
    DatabaseMigrationRollbackResult, DatabaseMigrationRunResult, DatabaseProvisioningResult,
    DatabaseRestoreResult, ProjectDatabaseProfile,
};
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project::Project;
use crate::domain::project::project_id::ProjectId;
use crate::shared::result::app_result::AppResult;

pub trait DatabaseProvisioner: Send + Sync {
    fn provision_project_database(
        &self,
        project: &Project,
        database_type: DatabaseType,
    ) -> AppResult<DatabaseProvisioningResult>;

    fn backup_project_database(
        &self,
        profile: &ProjectDatabaseProfile,
        options: DatabaseBackupOptions,
    ) -> AppResult<DatabaseBackupResult>;

    fn restore_project_database(
        &self,
        profile: &ProjectDatabaseProfile,
        backup_path: &str,
    ) -> AppResult<DatabaseRestoreResult>;

    fn restore_project_database_with_replay(
        &self,
        profile: &ProjectDatabaseProfile,
        base_backup_path: &str,
        replay_source_path: &str,
        target_time: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AppResult<DatabaseContinuousReplayRestoreResult>;

    fn create_migration_file(
        &self,
        project_id: &ProjectId,
        database_type: DatabaseType,
        migration_dir: &str,
        name: &str,
    ) -> AppResult<DatabaseMigrationFile>;

    fn run_migrations(
        &self,
        profile: &ProjectDatabaseProfile,
    ) -> AppResult<DatabaseMigrationRunResult>;

    fn rollback_migrations(
        &self,
        profile: &ProjectDatabaseProfile,
        steps: u16,
    ) -> AppResult<DatabaseMigrationRollbackResult>;

    fn generate_migration_rollback(
        &self,
        profile: &ProjectDatabaseProfile,
        migration_path: &str,
    ) -> AppResult<DatabaseMigrationRollbackGenerationResult>;
}
