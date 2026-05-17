use crate::domain::database::database_config::{
    DatabaseBackupSchedulerInstallResult, DatabaseBackupSchedulerStatus,
};
use crate::shared::result::app_result::AppResult;

pub trait DatabaseBackupScheduler: Send + Sync {
    fn scheduler_status(&self) -> AppResult<DatabaseBackupSchedulerStatus>;

    fn install_scheduler(&self) -> AppResult<DatabaseBackupSchedulerInstallResult>;

    fn uninstall_scheduler(&self) -> AppResult<DatabaseBackupSchedulerInstallResult>;
}
