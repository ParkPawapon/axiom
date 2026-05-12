use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn validate_service_id(service_id: &str) -> AppResult<&str> {
    let trimmed = service_id.trim();

    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "service id must not be empty".to_string(),
        ));
    }

    if trimmed.len() > 64 {
        return Err(AppError::Validation(
            "service id must be 64 characters or fewer".to_string(),
        ));
    }

    let is_safe = trimmed
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');

    if !is_safe {
        return Err(AppError::Validation(
            "service id may only contain lowercase letters, numbers, and hyphens".to_string(),
        ));
    }

    Ok(trimmed)
}
