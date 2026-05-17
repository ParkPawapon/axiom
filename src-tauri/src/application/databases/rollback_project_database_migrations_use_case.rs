use chrono::Utc;

use crate::domain::database::database_config::DatabaseMigrationRollbackResult;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

use super::database_type_parser::parse_database_type;

const MAX_ROLLBACK_STEPS: u16 = 50;

pub fn rollback_project_database_migrations(
    database_repository: &dyn DatabaseProvisioningRepository,
    database_provisioner: &dyn DatabaseProvisioner,
    project_id: &str,
    database_type: &str,
    steps: u16,
) -> AppResult<DatabaseMigrationRollbackResult> {
    if steps == 0 || steps > MAX_ROLLBACK_STEPS {
        return Err(AppError::Validation(format!(
            "migration rollback steps must be between 1 and {MAX_ROLLBACK_STEPS}"
        )));
    }

    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let database_type = parse_database_type(database_type)?;
    let mut profile = database_repository
        .get_profile(&project_id, database_type)?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "{} database profile was not found for project `{}`",
                database_type.as_key(),
                project_id.0
            ))
        })?;
    let result = database_provisioner.rollback_migrations(&profile, steps)?;

    if !result.rolled_back_migrations.is_empty() {
        profile.applied_migrations.retain(|applied| {
            !result
                .rolled_back_migrations
                .iter()
                .any(|rolled_back| rolled_back == applied)
        });
        profile.updated_at = Utc::now();
        database_repository.save_profile(profile)?;
    }

    Ok(result)
}
