use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use directories::ProjectDirs;

use crate::domain::settings::app_settings::AppSettings;
use crate::ports::settings_repository::SettingsRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Clone)]
pub struct LocalConfigRepository {
    settings_path: PathBuf,
}

impl LocalConfigRepository {
    pub fn new() -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application config directory".to_string())
        })?;

        Ok(Self::with_settings_path(
            project_dirs.config_dir().join("settings.json"),
        ))
    }

    pub fn with_settings_path(settings_path: PathBuf) -> Self {
        Self { settings_path }
    }

    fn ensure_parent_dir(&self) -> AppResult<()> {
        let Some(parent) = self.settings_path.parent() else {
            return Err(AppError::Configuration(
                "settings path must have a parent directory".to_string(),
            ));
        };

        fs::create_dir_all(parent).map_err(|error| {
            AppError::Infrastructure(format!("failed to create settings directory: {error}"))
        })
    }
}

impl SettingsRepository for LocalConfigRepository {
    fn read_settings(&self) -> AppResult<AppSettings> {
        if !self.settings_path.exists() {
            return Ok(AppSettings::default());
        }

        let contents = fs::read_to_string(&self.settings_path).map_err(|error| {
            AppError::Infrastructure(format!("failed to read settings file: {error}"))
        })?;

        serde_json::from_str(&contents)
            .map_err(|error| AppError::Configuration(format!("settings file is invalid: {error}")))
    }

    fn save_settings(&self, mut settings: AppSettings) -> AppResult<AppSettings> {
        self.ensure_parent_dir()?;
        settings.updated_at = Utc::now();

        let contents =
            serde_json::to_string_pretty(&settings).map_err(|_error| AppError::Unexpected)?;
        let temporary_path = temporary_settings_path(&self.settings_path);

        fs::write(&temporary_path, contents).map_err(|error| {
            AppError::Infrastructure(format!("failed to write temporary settings file: {error}"))
        })?;
        fs::rename(&temporary_path, &self.settings_path).map_err(|error| {
            let _ = fs::remove_file(&temporary_path);
            AppError::Infrastructure(format!(
                "failed to replace settings file atomically: {error}"
            ))
        })?;

        Ok(settings)
    }
}

fn temporary_settings_path(settings_path: &Path) -> PathBuf {
    let mut file_name = settings_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("settings.json")
        .to_string();
    file_name.push_str(".tmp");

    settings_path.with_file_name(file_name)
}
