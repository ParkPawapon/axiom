use crate::domain::project::project::Project;
use crate::domain::project::project_docker::{
    ProjectDockerActionResult, ProjectDockerComposeConfig, ProjectDockerStatus,
};
use crate::domain::runtime::runtime_version::RuntimeVersion;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DockerEngineProbe {
    pub cli_found: bool,
    pub engine_running: bool,
    pub compose_project_count: Option<usize>,
    pub status_message: String,
}

pub trait DockerClient: Send + Sync {
    fn probe_engine(&self) -> AppResult<DockerEngineProbe>;

    fn start_configured_compose_project(&self) -> AppResult<Option<String>>;

    fn stop_configured_compose_project(&self) -> AppResult<Option<String>>;

    fn generate_project_compose(
        &self,
        project: &Project,
        php_version: &RuntimeVersion,
    ) -> AppResult<ProjectDockerComposeConfig>;

    fn get_project_status(&self, project: &Project) -> AppResult<ProjectDockerStatus>;

    fn start_project(
        &self,
        config: &ProjectDockerComposeConfig,
    ) -> AppResult<ProjectDockerActionResult>;

    fn stop_project(&self, project: &Project) -> AppResult<ProjectDockerActionResult>;

    fn restart_project(
        &self,
        config: &ProjectDockerComposeConfig,
    ) -> AppResult<ProjectDockerActionResult>;
}
