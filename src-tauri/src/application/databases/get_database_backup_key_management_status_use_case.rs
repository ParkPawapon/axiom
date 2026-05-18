use crate::domain::database::database_config::DatabaseBackupKeyManagementStatus;
use crate::infrastructure::databases::backup_artifacts::backup_key_management_status;
use crate::ports::secure_storage::SecureStorage;
use crate::shared::result::app_result::AppResult;

pub fn get_database_backup_key_management_status(
    secure_storage: &dyn SecureStorage,
) -> AppResult<DatabaseBackupKeyManagementStatus> {
    backup_key_management_status(secure_storage)
}
