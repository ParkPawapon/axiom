use crate::domain::database::database_config::DatabaseBackupTrustExportResult;
use crate::infrastructure::databases::backup_artifacts::export_backup_trust_bundle;
use crate::ports::secure_storage::SecureStorage;
use crate::shared::result::app_result::AppResult;

pub fn export_database_backup_trust_bundle(
    secure_storage: &dyn SecureStorage,
    output_dir: &str,
) -> AppResult<DatabaseBackupTrustExportResult> {
    export_backup_trust_bundle(secure_storage, output_dir)
}
