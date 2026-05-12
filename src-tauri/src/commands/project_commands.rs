use tauri::State;

use crate::application::projects::get_project_php_version_use_case;
use crate::application::projects::request_project_php_install_use_case;
use crate::application::projects::select_project_php_version_use_case;
use crate::bootstrap::app_state::AppState;
use crate::domain::project::project_php_version::{ProjectPhpInstallPlan, ProjectPhpVersionConfig};
use crate::shared::error::command_error_mapper::{map_command_error, CommandErrorPayload};

#[tauri::command]
pub fn get_project_php_version(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectPhpVersionConfig, CommandErrorPayload> {
    get_project_php_version_use_case::get_project_php_version(
        state.project_runtime_repository(),
        state.php_runtime_detector(),
        &project_id,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project PHP version read command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn select_project_php_version(
    state: State<'_, AppState>,
    project_id: String,
    php_version: String,
) -> Result<ProjectPhpVersionConfig, CommandErrorPayload> {
    select_project_php_version_use_case::select_project_php_version(
        state.project_runtime_repository(),
        state.php_runtime_detector(),
        &project_id,
        &php_version,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project PHP version selection command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn request_project_php_install(
    state: State<'_, AppState>,
    project_id: String,
    php_version: String,
) -> Result<ProjectPhpInstallPlan, CommandErrorPayload> {
    request_project_php_install_use_case::request_project_php_install(
        state.project_runtime_repository(),
        &project_id,
        &php_version,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project PHP install request command failed");
        map_command_error(&error)
    })
}
