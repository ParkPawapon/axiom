use crate::domain::database::database_config::DatabaseBackupArtifactTrustEnrollmentResult;
use crate::infrastructure::databases::backup_artifacts::enroll_backup_artifact_trust;
use crate::ports::secure_storage::SecureStorage;
use crate::shared::result::app_result::AppResult;

pub fn enroll_database_backup_artifact_trust(
    secure_storage: &dyn SecureStorage,
    backup_path: &str,
) -> AppResult<DatabaseBackupArtifactTrustEnrollmentResult> {
    enroll_backup_artifact_trust(secure_storage, backup_path)
}
