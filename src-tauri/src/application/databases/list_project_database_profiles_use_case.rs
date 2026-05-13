use crate::domain::database::database_config::ProjectDatabaseProfile;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_provisioning_repository::DatabaseProvisioningRepository;
use crate::ports::project_repository::ProjectRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn list_project_database_profiles(
    project_repository: &dyn ProjectRepository,
    database_repository: &dyn DatabaseProvisioningRepository,
    project_id: &str,
) -> AppResult<Vec<ProjectDatabaseProfile>> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    ensure_project_exists(project_repository, &project_id)?;

    database_repository.list_profiles(&project_id)
}

fn ensure_project_exists(
    project_repository: &dyn ProjectRepository,
    project_id: &ProjectId,
) -> AppResult<()> {
    project_repository
        .get_project(project_id)?
        .map(|_| ())
        .ok_or_else(|| AppError::NotFound(format!("project `{}` was not found", project_id.0)))
}
