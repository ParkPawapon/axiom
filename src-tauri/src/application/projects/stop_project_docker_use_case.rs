use crate::domain::project::project_docker::ProjectDockerActionResult;
use crate::ports::docker_client::DockerClient;
use crate::ports::project_repository::ProjectRepository;
use crate::shared::result::app_result::AppResult;

use super::docker_support::load_project;

pub fn stop_project_docker(
    project_repository: &dyn ProjectRepository,
    docker_client: &dyn DockerClient,
    project_id: &str,
) -> AppResult<ProjectDockerActionResult> {
    let project = load_project(project_repository, project_id)?;

    docker_client.stop_project(&project)
}
