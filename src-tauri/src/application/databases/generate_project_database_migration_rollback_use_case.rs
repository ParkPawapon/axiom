use crate::domain::database::database_config::DatabaseMigrationRollbackGenerationResult;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

use super::database_type_parser::parse_database_type;

pub fn generate_project_database_migration_rollback(
    database_repository: &dyn DatabaseProvisioningRepository,
    database_provisioner: &dyn DatabaseProvisioner,
    project_id: &str,
    database_type: &str,
    migration_path: &str,
) -> AppResult<DatabaseMigrationRollbackGenerationResult> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let database_type = parse_database_type(database_type)?;
    let profile = database_repository
        .get_profile(&project_id, database_type)?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "{} database profile was not found for project `{}`",
                database_type.as_key(),
                project_id.0
            ))
        })?;

    database_provisioner.generate_migration_rollback(&profile, migration_path)
}
