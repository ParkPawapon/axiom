use chrono::Utc;

use crate::domain::system::system_status::DockerCheckResult;
use crate::ports::docker_project_orchestrator::DockerProjectOrchestrator;
use crate::shared::result::app_result::AppResult;

pub fn check_docker(
    docker_project_orchestrator: &dyn DockerProjectOrchestrator,
) -> AppResult<DockerCheckResult> {
    let diagnostics = docker_project_orchestrator.diagnostics()?;
    let status_message = diagnostics.status_message.clone();

    Ok(DockerCheckResult {
        diagnostics,
        checked_at: Utc::now(),
        status_message,
    })
}
