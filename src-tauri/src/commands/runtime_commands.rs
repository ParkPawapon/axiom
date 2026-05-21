use tauri::State;

use crate::application::runtimes::detect_php_runtimes_use_case;
use crate::application::runtimes::validate_runtime_use_case;
use crate::bootstrap::app_state::AppState;
use crate::domain::runtime::php_runtime::PhpRuntime;
use crate::domain::runtime::runtime_validation::RuntimeValidationResult;
use crate::shared::error::command_error_mapper::{map_command_error, CommandErrorPayload};

#[tauri::command]
pub fn detect_php_runtimes(
    state: State<'_, AppState>,
) -> Result<Vec<PhpRuntime>, CommandErrorPayload> {
    detect_php_runtimes_use_case::detect_php_runtimes(state.php_runtime_detector()).map_err(
        |error| {
            tracing::warn!(?error, "PHP runtime detection command failed");
            map_command_error(&error)
        },
    )
}

#[tauri::command]
pub fn validate_runtime(
    state: State<'_, AppState>,
    php_version: String,
) -> Result<RuntimeValidationResult, CommandErrorPayload> {
    validate_runtime_use_case::validate_runtime(state.php_runtime_detector(), &php_version).map_err(
        |error| {
            tracing::warn!(?error, "PHP runtime validation command failed");
            map_command_error(&error)
        },
    )
}
