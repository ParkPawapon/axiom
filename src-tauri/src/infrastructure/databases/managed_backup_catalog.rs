use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::domain::database::database_config::{DatabaseBackupMetadata, ProjectDatabaseProfile};
use crate::ports::database_backup_catalog::DatabaseBackupCatalog;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Clone, Default)]
pub struct ManagedDatabaseBackupCatalog;

impl ManagedDatabaseBackupCatalog {
    pub fn new() -> Self {
        Self
    }
}

impl DatabaseBackupCatalog for ManagedDatabaseBackupCatalog {
    fn latest_backup_at_or_before(
        &self,
        profile: &ProjectDatabaseProfile,
        target_time: DateTime<Utc>,
    ) -> AppResult<Option<DatabaseBackupMetadata>> {
        let backup_dir = Path::new(&profile.backup_dir)
            .canonicalize()
            .map_err(|error| {
                AppError::Validation(format!(
                    "backup directory must exist before point-in-time restore: {error}"
                ))
            })?;
        let mut candidates = Vec::new();

        for entry in fs::read_dir(&backup_dir).map_err(|error| {
            AppError::Infrastructure(format!("failed to read managed backup catalog: {error}"))
        })? {
            let path = entry
                .map_err(|error| {
                    AppError::Infrastructure(format!(
                        "failed to inspect managed backup catalog entry: {error}"
                    ))
                })?
                .path();

            if !is_metadata_file(&path) {
                continue;
            }

            let metadata = read_metadata(&path)?;
            if metadata.project_id != profile.project_id
                || metadata.database_type != profile.database_type
                || metadata.created_at > target_time
            {
                continue;
            }

            if Path::new(&metadata.backup_path).is_file() {
                candidates.push(metadata);
            }
        }

        candidates.sort_by_key(|metadata| metadata.created_at);
        Ok(candidates.pop())
    }
}

fn read_metadata(path: &Path) -> AppResult<DatabaseBackupMetadata> {
    let contents = fs::read_to_string(path).map_err(|error| {
        AppError::Infrastructure(format!("failed to read managed backup metadata: {error}"))
    })?;

    serde_json::from_str(&contents).map_err(|error| {
        AppError::Configuration(format!("managed backup metadata is invalid: {error}"))
    })
}

fn is_metadata_file(path: &Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .is_some_and(|file_name| file_name.ends_with(".metadata.json"))
}
