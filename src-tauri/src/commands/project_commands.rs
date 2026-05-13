use tauri::State;

use crate::application::projects::create_project_use_case;
use crate::application::projects::delete_project_use_case;
use crate::application::projects::get_project_php_process_status_use_case;
use crate::application::projects::get_project_php_version_use_case;
use crate::application::projects::get_project_use_case;
use crate::application::projects::install_project_php_runtime_use_case;
use crate::application::projects::list_projects_use_case;
use crate::application::projects::request_project_php_install_use_case;
use crate::application::projects::select_project_php_version_use_case;
use crate::application::projects::start_project_php_process_use_case;
use crate::application::projects::stop_project_php_process_use_case;
use crate::application::projects::update_project_use_case;
use crate::application::projects::validate_project_path_use_case;
use crate::bootstrap::app_state::AppState;
use crate::domain::project::project::Project;
use crate::domain::project::project_path::ProjectPath;
use crate::domain::project::project_php_version::{
    ProjectPhpInstallPlan, ProjectPhpInstallResult, ProjectPhpVersionConfig,
};
use crate::domain::project::project_process::ProjectPhpProcessStatus;
use crate::shared::error::command_error_mapper::{map_command_error, CommandErrorPayload};

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, CommandErrorPayload> {
    list_projects_use_case::list_projects(state.project_repository()).map_err(|error| {
        tracing::warn!(?error, "project list command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn get_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Project, CommandErrorPayload> {
    get_project_use_case::get_project(state.project_repository(), &project_id).map_err(|error| {
        tracing::warn!(?error, "project read command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn create_project(
    state: State<'_, AppState>,
    name: String,
    document_root: String,
) -> Result<Project, CommandErrorPayload> {
    create_project_use_case::create_project(state.project_repository(), &name, &document_root)
        .map_err(|error| {
            tracing::warn!(?error, "project create command failed");
            map_command_error(&error)
        })
}

#[tauri::command]
pub fn update_project(
    state: State<'_, AppState>,
    project_id: String,
    name: String,
    document_root: String,
) -> Result<Project, CommandErrorPayload> {
    update_project_use_case::update_project(
        state.project_repository(),
        &project_id,
        &name,
        &document_root,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project update command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn delete_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<(), CommandErrorPayload> {
    delete_project_use_case::delete_project(state.project_repository(), &project_id).map_err(
        |error| {
            tracing::warn!(?error, "project delete command failed");
            map_command_error(&error)
        },
    )
}

#[tauri::command]
pub fn validate_project_path(document_root: String) -> Result<ProjectPath, CommandErrorPayload> {
    validate_project_path_use_case::validate_project_path(&document_root).map_err(|error| {
        tracing::warn!(?error, "project path validation command failed");
        map_command_error(&error)
    })
}

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

#[tauri::command]
pub fn install_project_php_runtime(
    state: State<'_, AppState>,
    project_id: String,
    php_version: String,
) -> Result<ProjectPhpInstallResult, CommandErrorPayload> {
    install_project_php_runtime_use_case::install_project_php_runtime(
        state.project_runtime_repository(),
        state.php_runtime_detector(),
        state.php_runtime_installer(),
        &project_id,
        &php_version,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project PHP runtime install command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn get_project_php_process_status(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectPhpProcessStatus, CommandErrorPayload> {
    get_project_php_process_status_use_case::get_project_php_process_status(
        state.project_php_process_manager(),
        &project_id,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project PHP process status command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn start_project_php_process(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectPhpProcessStatus, CommandErrorPayload> {
    start_project_php_process_use_case::start_project_php_process(
        state.project_runtime_repository(),
        state.php_runtime_detector(),
        state.project_php_process_manager(),
        &project_id,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project PHP process start command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn stop_project_php_process(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectPhpProcessStatus, CommandErrorPayload> {
    stop_project_php_process_use_case::stop_project_php_process(
        state.project_php_process_manager(),
        &project_id,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project PHP process stop command failed");
        map_command_error(&error)
    })
}
