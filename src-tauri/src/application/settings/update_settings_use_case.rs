use std::path::Path;

use crate::domain::settings::app_settings::{AppSettingsUpdate, AppSettingsUpdateResult};
use crate::ports::settings_repository::SettingsRepository;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_port::validate_port;

const MAX_PHP_SEARCH_PATHS: usize = 24;
const MAX_DOCKER_CONTEXT_LENGTH: usize = 80;

pub fn update_settings(
    settings_repository: &dyn SettingsRepository,
    update: AppSettingsUpdate,
) -> AppResult<AppSettingsUpdateResult> {
    let mut settings = settings_repository.read_settings()?;

    if let Some(theme) = update.theme {
        settings.theme = theme;
    }

    if let Some(paths) = update.php_binary_search_paths {
        settings.php_binary_search_paths = validate_php_binary_search_paths(paths)?;
    }

    if let Some(docker_context) = update.docker_context {
        settings.docker_context = docker_context
            .map(|context| validate_docker_context(&context))
            .transpose()?;
    }

    if let Some(port) = update.default_php_port {
        settings.default_php_port = validate_port(port)?;
    }

    if let Some(port) = update.default_mysql_port {
        settings.default_mysql_port = validate_port(port)?;
    }

    if let Some(port) = update.default_postgres_port {
        settings.default_postgres_port = validate_port(port)?;
    }

    if let Some(audit_log_enabled) = update.audit_log_enabled {
        settings.audit_log_enabled = audit_log_enabled;
    }

    let settings = settings_repository.save_settings(settings)?;

    Ok(AppSettingsUpdateResult {
        settings,
        status_message: "Application settings were validated and persisted.".to_string(),
    })
}

fn validate_php_binary_search_paths(paths: Vec<String>) -> AppResult<Vec<String>> {
    if paths.len() > MAX_PHP_SEARCH_PATHS {
        return Err(AppError::Validation(format!(
            "PHP binary search paths must contain {MAX_PHP_SEARCH_PATHS} entries or fewer"
        )));
    }

    let mut normalized_paths = Vec::with_capacity(paths.len());

    for path in paths {
        let path = path.trim();

        if path.is_empty() {
            continue;
        }

        if path.as_bytes().contains(&0) || path.chars().any(char::is_control) {
            return Err(AppError::Validation(
                "PHP binary search paths must not contain null bytes or control characters"
                    .to_string(),
            ));
        }

        let path = Path::new(path);

        if !path.is_absolute() {
            return Err(AppError::Validation(
                "PHP binary search paths must be absolute paths".to_string(),
            ));
        }

        normalized_paths.push(path.to_string_lossy().into_owned());
    }

    normalized_paths.sort();
    normalized_paths.dedup();

    Ok(normalized_paths)
}

fn validate_docker_context(context: &str) -> AppResult<String> {
    let context = context.trim();

    if context.is_empty() {
        return Err(AppError::Validation(
            "Docker context must not be empty when provided".to_string(),
        ));
    }

    if context.len() > MAX_DOCKER_CONTEXT_LENGTH {
        return Err(AppError::Validation(format!(
            "Docker context must be {MAX_DOCKER_CONTEXT_LENGTH} characters or fewer"
        )));
    }

    if !context
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(AppError::Validation(
            "Docker context may only contain letters, numbers, dots, underscores, and hyphens"
                .to_string(),
        ));
    }

    Ok(context.to_string())
}
