use tauri::State;

use crate::application::logs::read_logs_use_case;
use crate::bootstrap::app_state::AppState;
use crate::domain::logs::project_log::ProjectLogReadResult;
use crate::shared::error::command_error_mapper::{map_command_error, CommandErrorPayload};

#[tauri::command]
pub fn read_project_logs(
    state: State<'_, AppState>,
    project_id: String,
    max_lines: Option<usize>,
    query: Option<String>,
) -> Result<ProjectLogReadResult, CommandErrorPayload> {
    read_logs_use_case::read_project_logs(
        state.project_repository(),
        state.log_reader(),
        &project_id,
        max_lines,
        query,
    )
    .map_err(|error| {
        tracing::warn!(?error, "project logs read command failed");
        map_command_error(&error)
    })
}
