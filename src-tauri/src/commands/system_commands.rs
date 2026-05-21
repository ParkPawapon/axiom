use tauri::State;

use crate::application::system::check_docker_use_case;
use crate::application::system::check_permissions_use_case;
use crate::application::system::check_port_use_case;
use crate::bootstrap::app_state::AppState;
use crate::domain::system::system_status::{
    DockerCheckResult, PermissionCheckResult, PortCheckResult,
};
use crate::shared::error::command_error_mapper::{map_command_error, CommandErrorPayload};

#[tauri::command]
pub fn check_port(
    state: State<'_, AppState>,
    port: u16,
) -> Result<PortCheckResult, CommandErrorPayload> {
    check_port_use_case::check_port(state.port_scanner(), port).map_err(|error| {
        tracing::warn!(?error, "port check command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn check_docker(state: State<'_, AppState>) -> Result<DockerCheckResult, CommandErrorPayload> {
    check_docker_use_case::check_docker(state.docker_project_orchestrator()).map_err(|error| {
        tracing::warn!(?error, "Docker check command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn check_permissions(
    state: State<'_, AppState>,
) -> Result<PermissionCheckResult, CommandErrorPayload> {
    check_permissions_use_case::check_permissions(state.permission_manager()).map_err(|error| {
        tracing::warn!(?error, "permission check command failed");
        map_command_error(&error)
    })
}
