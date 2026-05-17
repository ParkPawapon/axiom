use crate::domain::project::project_docker::ProjectDockerActionResult;
use crate::ports::docker_client::DockerClient;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::project_runtime_repository::ProjectRuntimeRepository;
use crate::shared::result::app_result::AppResult;

use super::docker_support::{ensure_project_compose, load_project};

pub fn restart_project_docker(
    project_repository: &dyn ProjectRepository,
    runtime_repository: &dyn ProjectRuntimeRepository,
    docker_client: &dyn DockerClient,
    project_id: &str,
) -> AppResult<ProjectDockerActionResult> {
    let project = load_project(project_repository, project_id)?;
    let config = ensure_project_compose(docker_client, runtime_repository, &project)?;

    docker_client.restart_project(&config)
}
