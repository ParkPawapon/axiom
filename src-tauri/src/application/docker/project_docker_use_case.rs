use crate::domain::docker::docker_project::{
    DockerComposeProfile, DockerDiagnosticsReport, DockerProjectActionResult,
    DockerProjectComposePlan, DockerProjectLogReadResult, DockerProjectRuntimeStatus,
    DockerProjectVolumeLifecycleResult,
};
use crate::domain::project::project::Project;
use crate::domain::project::project_id::ProjectId;
use crate::ports::docker_project_orchestrator::DockerProjectOrchestrator;
use crate::ports::project_repository::ProjectRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_project_id::validate_project_id;

pub fn get_docker_diagnostics(
    orchestrator: &dyn DockerProjectOrchestrator,
) -> AppResult<DockerDiagnosticsReport> {
    orchestrator.diagnostics()
}

pub fn generate_project_docker_compose(
    project_repository: &dyn ProjectRepository,
    orchestrator: &dyn DockerProjectOrchestrator,
    project_id: &str,
    profiles: &[DockerComposeProfile],
) -> AppResult<DockerProjectComposePlan> {
    let project = resolve_project(project_repository, project_id)?;

    orchestrator.generate_compose_plan(&project, profiles)
}

pub fn get_project_docker_status(
    project_repository: &dyn ProjectRepository,
    orchestrator: &dyn DockerProjectOrchestrator,
    project_id: &str,
) -> AppResult<DockerProjectRuntimeStatus> {
    let project = resolve_project(project_repository, project_id)?;

    orchestrator.get_runtime_status(&project)
}

pub fn start_project_docker_services(
    project_repository: &dyn ProjectRepository,
    orchestrator: &dyn DockerProjectOrchestrator,
    project_id: &str,
    profiles: &[DockerComposeProfile],
) -> AppResult<DockerProjectActionResult> {
    let project = resolve_project(project_repository, project_id)?;

    orchestrator.start_project(&project, profiles)
}

pub fn stop_project_docker_services(
    project_repository: &dyn ProjectRepository,
    orchestrator: &dyn DockerProjectOrchestrator,
    project_id: &str,
) -> AppResult<DockerProjectActionResult> {
    let project = resolve_project(project_repository, project_id)?;

    orchestrator.stop_project(&project)
}

pub fn restart_project_docker_services(
    project_repository: &dyn ProjectRepository,
    orchestrator: &dyn DockerProjectOrchestrator,
    project_id: &str,
    profiles: &[DockerComposeProfile],
) -> AppResult<DockerProjectActionResult> {
    let project = resolve_project(project_repository, project_id)?;

    orchestrator.restart_project(&project, profiles)
}

pub fn ensure_project_docker_volumes(
    project_repository: &dyn ProjectRepository,
    orchestrator: &dyn DockerProjectOrchestrator,
    project_id: &str,
    profiles: &[DockerComposeProfile],
) -> AppResult<DockerProjectVolumeLifecycleResult> {
    let project = resolve_project(project_repository, project_id)?;

    orchestrator.ensure_project_volumes(&project, profiles)
}

pub fn remove_project_docker_volumes(
    project_repository: &dyn ProjectRepository,
    orchestrator: &dyn DockerProjectOrchestrator,
    project_id: &str,
) -> AppResult<DockerProjectVolumeLifecycleResult> {
    let project = resolve_project(project_repository, project_id)?;

    orchestrator.remove_project_volumes(&project)
}

pub fn read_project_docker_logs(
    project_repository: &dyn ProjectRepository,
    orchestrator: &dyn DockerProjectOrchestrator,
    project_id: &str,
    tail_lines: u16,
) -> AppResult<DockerProjectLogReadResult> {
    let project = resolve_project(project_repository, project_id)?;

    orchestrator.read_project_logs(&project, tail_lines)
}

fn resolve_project(
    project_repository: &dyn ProjectRepository,
    project_id: &str,
) -> AppResult<Project> {
    let project_id = ProjectId(validate_project_id(project_id)?.to_string());

    project_repository
        .get_project(&project_id)?
        .ok_or_else(|| AppError::NotFound(format!("project `{}` was not found", project_id.0)))
}
