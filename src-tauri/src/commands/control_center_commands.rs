use tauri::State;

use crate::application::control_center::get_control_center_summary_use_case::{
    get_control_center_summary as get_summary, ControlCenterDependencies,
};
use crate::application::control_center::get_setup_diagnostics_use_case::get_setup_diagnostics as get_diagnostics;
use crate::bootstrap::app_state::AppState;
use crate::domain::control_center::control_center_summary::ControlCenterSummary;
use crate::domain::control_center::setup_diagnostic::SetupDiagnosticsReport;
use crate::shared::error::command_error_mapper::{map_command_error, CommandErrorPayload};

#[tauri::command]
pub fn get_control_center_summary(
    state: State<'_, AppState>,
    selected_project_id: Option<String>,
) -> Result<ControlCenterSummary, CommandErrorPayload> {
    get_summary(dependencies(&state), selected_project_id).map_err(|error| {
        tracing::warn!(?error, "control center summary command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn get_setup_diagnostics(
    state: State<'_, AppState>,
    selected_project_id: Option<String>,
) -> Result<SetupDiagnosticsReport, CommandErrorPayload> {
    get_diagnostics(dependencies(&state), selected_project_id).map_err(|error| {
        tracing::warn!(?error, "setup diagnostics command failed");
        map_command_error(&error)
    })
}

fn dependencies<'a>(state: &'a State<'_, AppState>) -> ControlCenterDependencies<'a> {
    ControlCenterDependencies {
        project_repository: state.project_repository(),
        project_runtime_repository: state.project_runtime_repository(),
        project_php_process_manager: state.project_php_process_manager(),
        database_repository: state.database_provisioning_repository(),
        docker_project_orchestrator: state.docker_project_orchestrator(),
        log_reader: state.log_reader(),
        port_scanner: state.port_scanner(),
        service_manager: state.service_manager(),
    }
}
