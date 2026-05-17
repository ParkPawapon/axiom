use chrono::Utc;

use crate::domain::database::database_config::{
    DatabaseBackupRemoteDestination, DatabaseBackupRemoteDestinationUpdate,
    DatabaseBackupRemoteDestinationUpdateResult,
};
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_backup_destination_repository::DatabaseBackupDestinationRepository;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

use super::database_type_parser::parse_database_type;

pub fn update_database_backup_destination(
    backup_destination_repository: &dyn DatabaseBackupDestinationRepository,
    database_repository: &dyn DatabaseProvisioningRepository,
    project_id: &str,
    database_type: &str,
    update: DatabaseBackupRemoteDestinationUpdate,
) -> AppResult<DatabaseBackupRemoteDestinationUpdateResult> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let database_type = parse_database_type(database_type)?;

    database_repository
        .get_profile(&project_id, database_type)?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "{} database profile was not found for project `{}`",
                database_type.as_key(),
                project_id.0
            ))
        })?;

    validate_destination_update(&update)?;

    let destination =
        backup_destination_repository.save_destination(DatabaseBackupRemoteDestination {
            project_id,
            database_type,
            enabled: update.enabled,
            destination_path: update.destination_path.trim().to_string(),
            updated_at: Utc::now(),
        })?;

    Ok(DatabaseBackupRemoteDestinationUpdateResult {
        destination,
        status_message: "Database backup destination was saved.".to_string(),
    })
}

fn validate_destination_update(update: &DatabaseBackupRemoteDestinationUpdate) -> AppResult<()> {
    if update.enabled && update.destination_path.trim().is_empty() {
        return Err(AppError::Validation(
            "backup destination path is required when remote backup is enabled".to_string(),
        ));
    }

    if !update.destination_path.trim().is_empty() {
        let path = std::path::Path::new(update.destination_path.trim());
        if !path.is_absolute() {
            return Err(AppError::Validation(
                "backup destination path must be absolute".to_string(),
            ));
        }
    }

    Ok(())
}
