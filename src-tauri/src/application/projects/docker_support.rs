use crate::domain::project::project::Project;
use crate::domain::project::project_docker::ProjectDockerComposeConfig;
use crate::domain::project::project_id::ProjectId;
use crate::domain::runtime::php_runtime::default_php_version;
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::ports::docker_client::DockerClient;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn load_project(
    project_repository: &dyn ProjectRepository,
    project_id: &str,
) -> AppResult<Project> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());

    project_repository
        .get_project(&project_id)?
        .ok_or_else(|| AppError::NotFound(format!("project `{}` was not found", project_id.0)))
}

pub fn selected_php_version(
    runtime_repository: &dyn ProjectRuntimeRepository,
    project_id: &ProjectId,
) -> AppResult<RuntimeVersion> {
    Ok(runtime_repository
        .get_php_selection(project_id)?
        .map(|selection| selection.php_version)
        .unwrap_or_else(default_php_version))
}

pub fn ensure_project_compose(
    docker_client: &dyn DockerClient,
    runtime_repository: &dyn ProjectRuntimeRepository,
    project: &Project,
) -> AppResult<ProjectDockerComposeConfig> {
    let php_version = selected_php_version(runtime_repository, &project.id)?;

    docker_client.generate_project_compose(project, &php_version)
}
