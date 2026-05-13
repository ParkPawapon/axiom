use crate::domain::project::project::Project;
use crate::domain::project::project_config::{CreateProjectRequest, UpdateProjectRequest};
use crate::domain::project::project_id::ProjectId;
use crate::shared::result::app_result::AppResult;

pub trait ProjectRepository: Send + Sync {
    fn list_projects(&self) -> AppResult<Vec<Project>>;

    fn get_project(&self, project_id: &ProjectId) -> AppResult<Option<Project>>;

    fn create_project(&self, request: CreateProjectRequest) -> AppResult<Project>;

    fn update_project(
        &self,
        project_id: &ProjectId,
        request: UpdateProjectRequest,
    ) -> AppResult<Project>;

    fn delete_project(&self, project_id: &ProjectId) -> AppResult<()>;
}
