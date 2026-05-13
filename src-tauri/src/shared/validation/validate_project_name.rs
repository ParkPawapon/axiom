use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn validate_project_name(name: &str) -> AppResult<&str> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "project name must not be empty".to_string(),
        ));
    }

    Ok(trimmed)
}
