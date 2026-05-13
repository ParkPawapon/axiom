use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::domain::database::database_config::ProjectDatabaseProfile;
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project_id::ProjectId;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProvisioningPaths {
    pub data_dir: PathBuf,
    pub backup_dir: PathBuf,
    pub migration_dir: PathBuf,
}

pub fn create_project_paths(
    storage_root: &Path,
    project_id: &ProjectId,
    database_type: DatabaseType,
) -> AppResult<ProvisioningPaths> {
    let root = storage_root
        .join(&project_id.0)
        .join(database_type.as_key());
    let paths = ProvisioningPaths {
        data_dir: root.join("data"),
        backup_dir: root.join("backups"),
        migration_dir: root.join("migrations"),
    };

    for directory in [&paths.data_dir, &paths.backup_dir, &paths.migration_dir] {
        fs::create_dir_all(directory).map_err(|error| {
            AppError::Infrastructure(format!(
                "failed to create database provisioning directory: {error}"
            ))
        })?;
        lock_directory_permissions(directory)?;
    }

    Ok(paths)
}

pub fn backup_path(profile: &ProjectDatabaseProfile) -> AppResult<PathBuf> {
    let backup_dir = validate_existing_directory(&profile.backup_dir, "backup directory")?;
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");

    Ok(backup_dir.join(format!(
        "{}_{}_{}.sql",
        profile.project_id.0,
        profile.database_type.as_key(),
        timestamp
    )))
}

pub fn collect_migration_files(migration_dir: &Path) -> AppResult<Vec<PathBuf>> {
    let mut migrations = Vec::new();

    for entry in fs::read_dir(migration_dir).map_err(|error| {
        AppError::Infrastructure(format!("failed to read migration directory: {error}"))
    })? {
        let path = entry
            .map_err(|error| {
                AppError::Infrastructure(format!("failed to inspect migration file: {error}"))
            })?
            .path();

        if path.extension().and_then(|extension| extension.to_str()) == Some("sql")
            && path.is_file()
        {
            migrations.push(path);
        }
    }

    migrations.sort();
    Ok(migrations)
}

pub fn validate_sql_file(path: &str) -> AppResult<PathBuf> {
    let path = Path::new(path.trim());

    if !path.is_absolute() {
        return Err(AppError::Validation(
            "backup path must be an absolute path".to_string(),
        ));
    }

    let canonical = path.canonicalize().map_err(|error| {
        AppError::Validation(format!("backup path must exist and be readable: {error}"))
    })?;

    if !canonical.is_file()
        || canonical
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("sql")
    {
        return Err(AppError::Validation(
            "backup path must point to a .sql file".to_string(),
        ));
    }

    Ok(canonical)
}

pub fn validate_existing_directory(path: &str, label: &str) -> AppResult<PathBuf> {
    let path = Path::new(path.trim());

    if !path.is_absolute() {
        return Err(AppError::Validation(format!("{label} must be absolute")));
    }

    let canonical = path.canonicalize().map_err(|error| {
        AppError::Validation(format!("{label} must exist and be readable: {error}"))
    })?;

    if !canonical.is_dir() {
        return Err(AppError::Validation(format!(
            "{label} must be an existing directory"
        )));
    }

    Ok(canonical)
}

#[cfg(unix)]
fn lock_directory_permissions(path: &Path) -> AppResult<()> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = fs::Permissions::from_mode(0o700);
    fs::set_permissions(path, permissions).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to lock database directory permissions: {error}"
        ))
    })
}

#[cfg(not(unix))]
fn lock_directory_permissions(_path: &Path) -> AppResult<()> {
    Ok(())
}
