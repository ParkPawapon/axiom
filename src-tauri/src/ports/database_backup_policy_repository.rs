use crate::domain::database::database_config::DatabaseBackupPolicy;
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project_id::ProjectId;
use crate::shared::result::app_result::AppResult;

pub trait DatabaseBackupPolicyRepository: Send + Sync {
    fn list_policies(&self, project_id: &ProjectId) -> AppResult<Vec<DatabaseBackupPolicy>>;

    fn list_all_policies(&self) -> AppResult<Vec<DatabaseBackupPolicy>>;

    fn get_policy(
        &self,
        project_id: &ProjectId,
        database_type: DatabaseType,
    ) -> AppResult<Option<DatabaseBackupPolicy>>;

    fn save_policy(&self, policy: DatabaseBackupPolicy) -> AppResult<DatabaseBackupPolicy>;
}
