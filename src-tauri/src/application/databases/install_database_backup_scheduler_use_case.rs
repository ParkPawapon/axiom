use crate::domain::database::database_config::DatabaseBackupSchedulerInstallResult;
use crate::ports::database_backup_scheduler::DatabaseBackupScheduler;
use crate::shared::result::app_result::AppResult;

pub fn install_database_backup_scheduler(
    scheduler: &dyn DatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerInstallResult> {
    scheduler.install_scheduler()
}
