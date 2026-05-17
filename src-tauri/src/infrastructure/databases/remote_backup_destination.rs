use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::database::database_config::{
    DatabaseBackupRemoteDestination, DatabaseBackupResult,
};
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn copy_backup_to_remote_destination(
    result: &DatabaseBackupResult,
    destination: &DatabaseBackupRemoteDestination,
) -> AppResult<Vec<String>> {
    if !destination.enabled {
        return Ok(Vec::new());
    }

    let destination_root = validate_destination_path(&destination.destination_path)?;
    let scoped_destination = destination_root
        .join(&destination.project_id.0)
        .join(destination.database_type.as_key());

    fs::create_dir_all(&scoped_destination).map_err(|error| {
        AppError::Infrastructure(format!("failed to create backup destination: {error}"))
    })?;

    let mut copied_paths = Vec::new();
    for source_path in backup_artifact_paths(result) {
        let copied_path = copy_one(&source_path, &scoped_destination)?;
        copied_paths.push(copied_path.to_string_lossy().into_owned());
    }

    Ok(copied_paths)
}

fn backup_artifact_paths(result: &DatabaseBackupResult) -> Vec<String> {
    let mut paths = vec![result.backup_path.clone()];

    if let Some(metadata_path) = &result.metadata_path {
        paths.push(metadata_path.clone());
    }

    if let Some(signature_path) = &result.signature_path {
        paths.push(signature_path.clone());
    }

    paths
}

fn validate_destination_path(path: &str) -> AppResult<PathBuf> {
    let path = Path::new(path.trim());

    if !path.is_absolute() {
        return Err(AppError::Validation(
            "backup destination path must be absolute".to_string(),
        ));
    }

    if path.exists() && !path.is_dir() {
        return Err(AppError::Validation(
            "backup destination path must be a directory".to_string(),
        ));
    }

    Ok(path.to_path_buf())
}

fn copy_one(source_path: &str, destination_dir: &Path) -> AppResult<PathBuf> {
    let source_path = Path::new(source_path);
    let source_path = source_path.canonicalize().map_err(|error| {
        AppError::Validation(format!(
            "backup artifact path must exist before copy: {error}"
        ))
    })?;
    let file_name = source_path
        .file_name()
        .ok_or_else(|| AppError::Validation("backup artifact has no file name".to_string()))?;
    let destination_path = destination_dir.join(file_name);

    fs::copy(&source_path, &destination_path).map_err(|error| {
        AppError::Infrastructure(format!("failed to copy backup artifact: {error}"))
    })?;

    Ok(destination_path)
}
