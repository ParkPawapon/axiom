use crate::domain::database::database_config::DatabaseBackupPolicy;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_backup_policy_repository::DatabaseBackupPolicyRepository;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn list_database_backup_policies(
    backup_policy_repository: &dyn DatabaseBackupPolicyRepository,
    project_id: &str,
) -> AppResult<Vec<DatabaseBackupPolicy>> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());

    backup_policy_repository.list_policies(&project_id)
}
