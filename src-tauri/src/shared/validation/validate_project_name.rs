use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn validate_project_name(name: &str) -> AppResult<&str> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("project name must not be empty".to_string()));
    }

    Ok(name)
}
