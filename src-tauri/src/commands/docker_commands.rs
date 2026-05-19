use tauri::State;

use crate::application::docker::project_docker_use_case;
use crate::bootstrap::app_state::AppState;
use crate::domain::docker::docker_project::{
    DockerComposeProfile, DockerDiagnosticsReport, DockerImagePinResolutionReport,
    DockerProjectActionResult, DockerProjectComposePlan, DockerProjectComposeRequest,
    DockerProjectImageOverride, DockerProjectLogReadResult, DockerProjectResourceLimits,
    DockerProjectRuntimeStatus, DockerProjectVolumeLifecycleResult,
};
use crate::domain::project::project_id::ProjectId;
use crate::shared::error::command_error_mapper::{map_command_error, CommandErrorPayload};

#[tauri::command]
pub fn get_docker_diagnostics(
    state: State<'_, AppState>,
) -> Result<DockerDiagnosticsReport, CommandErrorPayload> {
    project_docker_use_case::get_docker_diagnostics(state.docker_project_orchestrator()).map_err(
        |error| {
            tracing::warn!(?error, "docker diagnostics command failed");
            map_command_error(&error)
        },
    )
}

#[tauri::command]
pub fn generate_project_docker_compose(
    state: State<'_, AppState>,
    project_id: String,
    profiles: Vec<DockerComposeProfile>,
    image_overrides: Vec<DockerProjectImageOverride>,
    resource_limits: DockerProjectResourceLimits,
) -> Result<DockerProjectComposePlan, CommandErrorPayload> {
    let request = docker_request(project_id, profiles, image_overrides, resource_limits);

    project_docker_use_case::generate_project_docker_compose(
        state.project_repository(),
        state.docker_project_orchestrator(),
        &request,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project docker compose generation command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn resolve_project_docker_image_pins(
    state: State<'_, AppState>,
    project_id: String,
    profiles: Vec<DockerComposeProfile>,
    image_overrides: Vec<DockerProjectImageOverride>,
    resource_limits: DockerProjectResourceLimits,
) -> Result<DockerImagePinResolutionReport, CommandErrorPayload> {
    let request = docker_request(project_id, profiles, image_overrides, resource_limits);

    project_docker_use_case::resolve_project_docker_image_pins(
        state.docker_project_orchestrator(),
        &request,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project docker image pin resolution command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn get_project_docker_status(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<DockerProjectRuntimeStatus, CommandErrorPayload> {
    project_docker_use_case::get_project_docker_status(
        state.project_repository(),
        state.docker_project_orchestrator(),
        &project_id,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project docker status command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn start_project_docker_services(
    state: State<'_, AppState>,
    project_id: String,
    profiles: Vec<DockerComposeProfile>,
    image_overrides: Vec<DockerProjectImageOverride>,
    resource_limits: DockerProjectResourceLimits,
) -> Result<DockerProjectActionResult, CommandErrorPayload> {
    let request = docker_request(project_id, profiles, image_overrides, resource_limits);

    project_docker_use_case::start_project_docker_services(
        state.project_repository(),
        state.docker_project_orchestrator(),
        &request,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project docker start command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn stop_project_docker_services(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<DockerProjectActionResult, CommandErrorPayload> {
    project_docker_use_case::stop_project_docker_services(
        state.project_repository(),
        state.docker_project_orchestrator(),
        &project_id,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project docker stop command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn restart_project_docker_services(
    state: State<'_, AppState>,
    project_id: String,
    profiles: Vec<DockerComposeProfile>,
    image_overrides: Vec<DockerProjectImageOverride>,
    resource_limits: DockerProjectResourceLimits,
) -> Result<DockerProjectActionResult, CommandErrorPayload> {
    let request = docker_request(project_id, profiles, image_overrides, resource_limits);

    project_docker_use_case::restart_project_docker_services(
        state.project_repository(),
        state.docker_project_orchestrator(),
        &request,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project docker restart command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn ensure_project_docker_volumes(
    state: State<'_, AppState>,
    project_id: String,
    profiles: Vec<DockerComposeProfile>,
    image_overrides: Vec<DockerProjectImageOverride>,
    resource_limits: DockerProjectResourceLimits,
) -> Result<DockerProjectVolumeLifecycleResult, CommandErrorPayload> {
    let request = docker_request(project_id, profiles, image_overrides, resource_limits);

    project_docker_use_case::ensure_project_docker_volumes(
        state.project_repository(),
        state.docker_project_orchestrator(),
        &request,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project docker volume ensure command failed");
        map_command_error(&error)
    })
}

fn docker_request(
    project_id: String,
    profiles: Vec<DockerComposeProfile>,
    image_overrides: Vec<DockerProjectImageOverride>,
    resource_limits: DockerProjectResourceLimits,
) -> DockerProjectComposeRequest {
    DockerProjectComposeRequest {
        project_id: ProjectId(project_id),
        profiles,
        image_overrides,
        resource_limits,
    }
}

#[tauri::command]
pub fn remove_project_docker_volumes(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<DockerProjectVolumeLifecycleResult, CommandErrorPayload> {
    project_docker_use_case::remove_project_docker_volumes(
        state.project_repository(),
        state.docker_project_orchestrator(),
        &project_id,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project docker volume remove command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn read_project_docker_logs(
    state: State<'_, AppState>,
    project_id: String,
    tail_lines: u16,
) -> Result<DockerProjectLogReadResult, CommandErrorPayload> {
    project_docker_use_case::read_project_docker_logs(
        state.project_repository(),
        state.docker_project_orchestrator(),
        &project_id,
        tail_lines,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project docker logs command failed");
        map_command_error(&error)
    })
}
