use crate::domain::database::database_config::DatabaseBackupTrustImportResult;
use crate::infrastructure::databases::backup_artifacts::import_backup_trust_bundle;
use crate::ports::secure_storage::SecureStorage;
use crate::shared::result::app_result::AppResult;

pub fn import_database_backup_trust_bundle(
    secure_storage: &dyn SecureStorage,
    trust_bundle_path: &str,
) -> AppResult<DatabaseBackupTrustImportResult> {
    import_backup_trust_bundle(secure_storage, trust_bundle_path)
}
