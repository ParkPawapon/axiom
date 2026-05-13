use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn validate_project_name(name: &str) -> AppResult<&str> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "project name must not be empty".to_string(),
        ));
    }

    if trimmed.len() > 80 {
        return Err(AppError::Validation(
            "project name must be 80 characters or fewer".to_string(),
        ));
    }

    if trimmed
        .chars()
        .any(|character| character.is_control() || matches!(character, '/' | '\\'))
    {
        return Err(AppError::Validation(
            "project name must not contain control characters or path separators".to_string(),
        ));
    }

    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_like_project_names() {
        let result = validate_project_name("../site");

        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
