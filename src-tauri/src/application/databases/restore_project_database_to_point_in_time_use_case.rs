use chrono::{DateTime, Utc};

use crate::domain::database::database_config::DatabasePointInTimeRestoreResult;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_backup_catalog::DatabaseBackupCatalog;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

use super::database_type_parser::parse_database_type;

pub fn restore_project_database_to_point_in_time(
    database_repository: &dyn DatabaseProvisioningRepository,
    backup_catalog: &dyn DatabaseBackupCatalog,
    database_provisioner: &dyn DatabaseProvisioner,
    project_id: &str,
    database_type: &str,
    target_time: &str,
) -> AppResult<DatabasePointInTimeRestoreResult> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let database_type = parse_database_type(database_type)?;
    let target_time = parse_target_time(target_time)?;

    if target_time > Utc::now() {
        return Err(AppError::Validation(
            "point-in-time restore target cannot be in the future".to_string(),
        ));
    }

    let profile = database_repository
        .get_profile(&project_id, database_type)?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "{} database profile was not found for project `{}`",
                database_type.as_key(),
                project_id.0
            ))
        })?;
    let metadata = backup_catalog
        .latest_backup_at_or_before(&profile, target_time)?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "no managed {} backup was found at or before {}",
                database_type.as_key(),
                target_time.to_rfc3339()
            ))
        })?;
    let restore = database_provisioner.restore_project_database(&profile, &metadata.backup_path)?;

    Ok(DatabasePointInTimeRestoreResult {
        project_id,
        database_type,
        target_time,
        selected_backup_path: metadata.backup_path,
        selected_backup_created_at: metadata.created_at,
        restore,
        status_message: format!(
            "Point-in-time restore selected the latest managed backup at or before {}.",
            target_time.to_rfc3339()
        ),
    })
}

fn parse_target_time(target_time: &str) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(target_time.trim())
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| {
            AppError::Validation(format!(
                "point-in-time restore target must be an RFC3339 timestamp: {error}"
            ))
        })
}
