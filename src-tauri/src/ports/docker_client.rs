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
}
