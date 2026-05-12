use tauri::State;

use crate::application::services::get_service_status_use_case;
use crate::application::services::list_services_use_case;
use crate::application::services::restart_service_use_case;
use crate::application::services::start_service_use_case;
use crate::application::services::stop_service_use_case;
use crate::bootstrap::app_state::AppState;
use crate::domain::service::service::Service;
use crate::domain::service::service_action::ServiceActionOutcome;
use crate::shared::error::command_error_mapper::{map_command_error, CommandErrorPayload};

#[tauri::command]
pub fn list_services(state: State<'_, AppState>) -> Result<Vec<Service>, CommandErrorPayload> {
    list_services_use_case::list_services(state.service_manager()).map_err(|error| {
        tracing::warn!(?error, "service list command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn get_service_status(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<Service, CommandErrorPayload> {
    get_service_status_use_case::get_service_status(state.service_manager(), &service_id).map_err(
        |error| {
            tracing::warn!(?error, "service status command failed");
            map_command_error(&error)
        },
    )
}

#[tauri::command]
pub fn start_service(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<ServiceActionOutcome, CommandErrorPayload> {
    start_service_use_case::start_service(state.service_manager(), &service_id).map_err(|error| {
        tracing::warn!(?error, "service start command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn stop_service(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<ServiceActionOutcome, CommandErrorPayload> {
    stop_service_use_case::stop_service(state.service_manager(), &service_id).map_err(|error| {
        tracing::warn!(?error, "service stop command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn restart_service(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<ServiceActionOutcome, CommandErrorPayload> {
    restart_service_use_case::restart_service(state.service_manager(), &service_id).map_err(
        |error| {
            tracing::warn!(?error, "service restart command failed");
            map_command_error(&error)
        },
    )
}
