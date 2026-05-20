use tauri::State;

use crate::application::settings::read_settings_use_case;
use crate::application::settings::update_settings_use_case;
use crate::bootstrap::app_state::AppState;
use crate::domain::settings::app_settings::{
    AppSettings, AppSettingsUpdate, AppSettingsUpdateResult,
};
use crate::shared::error::command_error_mapper::{map_command_error, CommandErrorPayload};

#[tauri::command]
pub fn read_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandErrorPayload> {
    read_settings_use_case::read_settings(state.settings_repository()).map_err(|error| {
        tracing::warn!(?error, "settings read command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    update: AppSettingsUpdate,
) -> Result<AppSettingsUpdateResult, CommandErrorPayload> {
    update_settings_use_case::update_settings(state.settings_repository(), update).map_err(
        |error| {
            tracing::warn!(?error, "settings update command failed");
            map_command_error(&error)
        },
    )
}
