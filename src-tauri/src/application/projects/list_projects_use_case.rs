use crate::domain::project::project::Project;
use crate::ports::project_repository::ProjectRepository;
use crate::shared::result::app_result::AppResult;

pub fn list_projects(project_repository: &dyn ProjectRepository) -> AppResult<Vec<Project>> {
    project_repository.list_projects()
}
