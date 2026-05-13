use crate::domain::project::project_id::ProjectId;
use crate::ports::project_repository::ProjectRepository;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn delete_project(
    project_repository: &dyn ProjectRepository,
    project_id: &str,
) -> AppResult<()> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());

    project_repository.delete_project(&project_id)
}
