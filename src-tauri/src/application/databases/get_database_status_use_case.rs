use chrono::Utc;

use crate::domain::database::database_config::{ProjectDatabaseProfile, ProjectDatabaseStatus};
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project_id::ProjectId;
use crate::domain::service::service::Service;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::service_manager::ServiceManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

use super::database_type_parser::parse_database_type;

pub fn get_database_status(
    project_repository: &dyn ProjectRepository,
    database_repository: &dyn DatabaseProvisioningRepository,
    service_manager: &dyn ServiceManager,
    project_id: &str,
    database_type: &str,
) -> AppResult<ProjectDatabaseStatus> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let database_type = parse_database_type(database_type)?;

    project_repository
        .get_project(&project_id)?
        .ok_or_else(|| AppError::NotFound(format!("project `{}` was not found", project_id.0)))?;

    let profile = database_repository.get_profile(&project_id, database_type)?;
    let service = database_service_status(service_manager, database_type);
    let provisioned = profile.is_some();
    let status_message = status_message(database_type, profile.as_ref(), service.as_ref());

    Ok(ProjectDatabaseStatus {
        project_id,
        database_type,
        profile,
        service,
        provisioned,
        checked_at: Utc::now(),
        status_message,
    })
}

fn database_service_status(
    service_manager: &dyn ServiceManager,
    database_type: DatabaseType,
) -> Option<Service> {
    let service_id = match database_type {
        DatabaseType::Mysql => "mysql",
        DatabaseType::Postgresql => "postgresql",
    };

    service_manager.get_service_status(service_id).ok()
}

fn status_message(
    database_type: DatabaseType,
    profile: Option<&ProjectDatabaseProfile>,
    service: Option<&Service>,
) -> String {
    let database_name = match database_type {
        DatabaseType::Mysql => "MySQL",
        DatabaseType::Postgresql => "PostgreSQL",
    };
    let profile_message = if profile.is_some() {
        "profile is provisioned"
    } else {
        "profile is not provisioned"
    };
    let service_message = service
        .map(|service| format!("service status is {:?}", service.status))
        .unwrap_or_else(|| "service status is unavailable".to_string());

    format!("{database_name} {profile_message}; {service_message}.")
}
