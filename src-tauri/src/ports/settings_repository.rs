use crate::domain::settings::app_settings::AppSettings;
use crate::shared::result::app_result::AppResult;

pub trait SettingsRepository: Send + Sync {
    fn read_settings(&self) -> AppResult<AppSettings>;

    fn save_settings(&self, settings: AppSettings) -> AppResult<AppSettings>;
}
