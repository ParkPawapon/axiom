use crate::domain::database::database_config::DatabaseProvisioningResult;
use crate::domain::database::database_config::DatabaseProvisioningStatus;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::ports::project_repository::ProjectRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

use super::database_type_parser::parse_database_type;

pub fn provision_project_database(
    project_repository: &dyn ProjectRepository,
    database_repository: &dyn DatabaseProvisioningRepository,
    database_provisioner: &dyn DatabaseProvisioner,
    project_id: &str,
    database_type: &str,
) -> AppResult<DatabaseProvisioningResult> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let database_type = parse_database_type(database_type)?;
    let project = project_repository
        .get_project(&project_id)?
        .ok_or_else(|| AppError::NotFound(format!("project `{}` was not found", project_id.0)))?;

    if let Some(existing_profile) = database_repository.get_profile(&project_id, database_type)? {
        if existing_profile.status == DatabaseProvisioningStatus::Ready {
            return Ok(DatabaseProvisioningResult {
                profile: existing_profile,
                credential_stored: true,
                database_created: false,
                dependency_report: None,
                phpmyadmin_access: None,
                service_report: None,
                status_message: "Database profile is already provisioned.".to_string(),
            });
        }
    }

    let result = database_provisioner.provision_project_database(&project, database_type)?;
    database_repository.save_profile(result.profile.clone())?;

    Ok(result)
}
