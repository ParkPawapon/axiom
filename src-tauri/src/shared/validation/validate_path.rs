use std::path::{Path, PathBuf};

use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn validate_path(path: &str) -> AppResult<&str> {
    if path.trim().is_empty() {
        return Err(AppError::Validation("path must not be empty".to_string()));
    }

    Ok(path)
}

pub fn validate_existing_directory_path(path: &str) -> AppResult<PathBuf> {
    let trimmed = validate_path(path)?.trim();

    if trimmed.as_bytes().contains(&0) || trimmed.chars().any(char::is_control) {
        return Err(AppError::Validation(
            "path must not contain null bytes or control characters".to_string(),
        ));
    }

    let path = Path::new(trimmed);

    if !path.is_absolute() {
        return Err(AppError::Validation(
            "project document root must be an absolute path".to_string(),
        ));
    }

    let canonical = path.canonicalize().map_err(|error| {
        AppError::Validation(format!(
            "project document root must exist and be readable: {error}"
        ))
    })?;

    if !canonical.is_dir() {
        return Err(AppError::Validation(
            "project document root must be an existing directory".to_string(),
        ));
    }

    let parent = canonical.parent();

    if parent.is_none() {
        return Err(AppError::Validation(
            "project document root must not be the filesystem root".to_string(),
        ));
    }

    Ok(canonical)
}
