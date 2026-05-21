use crate::domain::settings::app_settings::AppSettings;
use crate::ports::settings_repository::SettingsRepository;
use crate::shared::result::app_result::AppResult;

pub fn read_settings(settings_repository: &dyn SettingsRepository) -> AppResult<AppSettings> {
    settings_repository.read_settings()
}
