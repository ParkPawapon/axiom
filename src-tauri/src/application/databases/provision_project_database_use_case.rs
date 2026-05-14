use crate::domain::database::database_config::DatabaseProvisioningResult;
use crate::domain::database::database_config::DatabaseProvisioningStatus;
use crate::domain::database::database_config::ManagedDatabaseServiceReport;
use crate::domain::project::project_id::ProjectId;
use crate::domain::service::service_action::ServiceActionState;
use crate::domain::service::service_status::ServiceStatus;
use crate::ports::database_dependency_manager::DatabaseDependencyManager;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::service_manager::ServiceManager;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

use super::database_type_parser::parse_database_type;

pub fn provision_project_database(
    project_repository: &dyn ProjectRepository,
    database_repository: &dyn DatabaseProvisioningRepository,
    database_dependency_manager: &dyn DatabaseDependencyManager,
    database_provisioner: &dyn DatabaseProvisioner,
    service_manager: &dyn ServiceManager,
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

    let dependency_report =
        database_dependency_manager.ensure_database_dependencies(database_type)?;
    let service_report = ensure_database_service_started(
        database_dependency_manager,
        service_manager,
        database_type,
    );
    let mut result = database_provisioner.provision_project_database(&project, database_type)?;
    result.dependency_report = Some(dependency_report);
    result.service_report = Some(service_report);
    database_repository.save_profile(result.profile.clone())?;

    Ok(result)
}

fn ensure_database_service_started(
    database_dependency_manager: &dyn DatabaseDependencyManager,
    service_manager: &dyn ServiceManager,
    database_type: crate::domain::database::database_type::DatabaseType,
) -> ManagedDatabaseServiceReport {
    let service_id = match database_type {
        crate::domain::database::database_type::DatabaseType::Mysql => "mysql",
        crate::domain::database::database_type::DatabaseType::Postgresql => "postgresql",
    };
    let mut messages = Vec::new();
    let mut started = false;

    match database_dependency_manager.start_database_service(database_type) {
        Ok(report) => {
            started |= report.started;
            messages.push(report.status_message);
        }
        Err(error) => messages.push(format!(
            "Package-manager service start could not complete automatically: {error}"
        )),
    }

    match service_manager.get_service_status(service_id) {
        Ok(service) if service.status == ServiceStatus::Running => {
            messages.push(format!("{} service is running.", service.name));
            return ManagedDatabaseServiceReport {
                service_id: service_id.to_string(),
                started,
                status_message: messages.join(" "),
            };
        }
        Ok(_) => match service_manager.start_service(service_id) {
            Ok(outcome) => {
                started |= outcome.state == ServiceActionState::Completed;
                messages.push(outcome.message);
            }
            Err(error) => messages.push(format!(
                "Service manager start could not complete automatically: {error}"
            )),
        },
        Err(error) => messages.push(format!(
            "Service manager status check could not complete automatically: {error}"
        )),
    }

    ManagedDatabaseServiceReport {
        service_id: service_id.to_string(),
        started,
        status_message: messages.join(" "),
    }
}
