use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn validate_project_id(project_id: &str) -> AppResult<&str> {
    let trimmed = project_id.trim();

    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "project id must not be empty".to_string(),
        ));
    }

    if trimmed.len() > 80 {
        return Err(AppError::Validation(
            "project id must be 80 characters or fewer".to_string(),
        ));
    }

    let is_safe = trimmed
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');

    if !is_safe {
        return Err(AppError::Validation(
            "project id may only contain lowercase letters, numbers, and hyphens".to_string(),
        ));
    }

    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_project_ids_with_path_like_characters() {
        let result = validate_project_id("../project");

        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
