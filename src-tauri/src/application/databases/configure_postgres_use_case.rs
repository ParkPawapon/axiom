use crate::domain::database::database_config::DatabaseProvisioningResult;
use crate::ports::database_dependency_manager::DatabaseDependencyManager;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::service_manager::ServiceManager;
use crate::shared::result::app_result::AppResult;

use super::provision_project_database_use_case::provision_project_database;

pub fn configure_postgres(
    project_repository: &dyn ProjectRepository,
    database_repository: &dyn DatabaseProvisioningRepository,
    database_dependency_manager: &dyn DatabaseDependencyManager,
    database_provisioner: &dyn DatabaseProvisioner,
    service_manager: &dyn ServiceManager,
    project_id: &str,
) -> AppResult<DatabaseProvisioningResult> {
    provision_project_database(
        project_repository,
        database_repository,
        database_dependency_manager,
        database_provisioner,
        service_manager,
        project_id,
        "postgresql",
    )
}
