use chrono::{DateTime, Utc};

use crate::domain::database::database_config::DatabaseContinuousReplayRestoreResult;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

use super::database_type_parser::parse_database_type;

pub fn restore_project_database_with_replay(
    database_repository: &dyn DatabaseProvisioningRepository,
    database_provisioner: &dyn DatabaseProvisioner,
    project_id: &str,
    database_type: &str,
    base_backup_path: &str,
    replay_source_path: &str,
    target_time: Option<String>,
) -> AppResult<DatabaseContinuousReplayRestoreResult> {
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
    let target_time = target_time
        .map(|value| {
            DateTime::parse_from_rfc3339(value.trim())
                .map(|value| value.with_timezone(&Utc))
                .map_err(|error| {
                    AppError::Validation(format!(
                        "recovery replay target time must be RFC3339: {error}"
                    ))
                })
        })
        .transpose()?;

    database_provisioner.restore_project_database_with_replay(
        &profile,
        base_backup_path,
        replay_source_path,
        target_time,
    )
}
