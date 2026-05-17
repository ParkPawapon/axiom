use crate::domain::database::database_config::{DatabaseBackupOptions, DatabaseBackupResult};
use crate::domain::project::project_id::ProjectId;
use crate::infrastructure::databases::remote_backup_destination::copy_backup_to_remote_destination;
use crate::ports::database_backup_destination_repository::DatabaseBackupDestinationRepository;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

use super::database_type_parser::parse_database_type;

pub fn backup_project_database(
    database_repository: &dyn DatabaseProvisioningRepository,
    backup_destination_repository: &dyn DatabaseBackupDestinationRepository,
    database_provisioner: &dyn DatabaseProvisioner,
    project_id: &str,
    database_type: &str,
    options: Option<DatabaseBackupOptions>,
) -> AppResult<DatabaseBackupResult> {
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

    let mut result =
        database_provisioner.backup_project_database(&profile, options.unwrap_or_default())?;

    if let Some(destination) =
        backup_destination_repository.get_destination(&project_id, database_type)?
    {
        result.remote_copy_paths = copy_backup_to_remote_destination(&result, &destination)?;
    }

    Ok(result)
}
