use crate::domain::project::project::Project;
use crate::domain::project::project_id::ProjectId;
use crate::ports::project_repository::ProjectRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn get_project(
    project_repository: &dyn ProjectRepository,
    project_id: &str,
) -> AppResult<Project> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());

    project_repository
        .get_project(&project_id)?
        .ok_or_else(|| AppError::NotFound(format!("project `{}` was not found", project_id.0)))
}
