use crate::domain::database::database_config::DatabaseBackupSchedulerStatus;
use crate::ports::database_backup_scheduler::DatabaseBackupScheduler;
use crate::shared::result::app_result::AppResult;

pub fn get_database_backup_scheduler_status(
    scheduler: &dyn DatabaseBackupScheduler,
) -> AppResult<DatabaseBackupSchedulerStatus> {
    scheduler.scheduler_status()
}
