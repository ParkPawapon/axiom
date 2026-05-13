use crate::domain::project::project::Project;
use crate::domain::project::project_config::UpdateProjectRequest;
use crate::domain::project::project_id::ProjectId;
use crate::domain::project::project_path::ProjectPath;
use crate::ports::project_repository::ProjectRepository;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_path::validate_existing_directory_path;
use crate::shared::validation::validate_project_id::validate_project_id;
use crate::shared::validation::validate_project_name::validate_project_name;

pub fn update_project(
    project_repository: &dyn ProjectRepository,
    project_id: &str,
    name: &str,
    document_root: &str,
) -> AppResult<Project> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());
    let name = validate_project_name(name)?.to_string();
    let document_root = validate_existing_directory_path(document_root)?;

    project_repository.update_project(
        &project_id,
        UpdateProjectRequest {
            name,
            document_root: ProjectPath(document_root.to_string_lossy().into_owned()),
        },
    )
}
