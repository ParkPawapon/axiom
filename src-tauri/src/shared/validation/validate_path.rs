use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn validate_non_empty_path(path: &str) -> AppResult<&str> {
    if path.trim().is_empty() {
        return Err(AppError::Validation("path must not be empty".to_string()));
    }

    Ok(path)
}
