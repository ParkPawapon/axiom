use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use directories::ProjectDirs;

use crate::domain::database::database_config::{
    DatabaseBackupResult, DatabaseMigrationFile, DatabaseMigrationRunResult,
    DatabaseProvisioningResult, DatabaseProvisioningStatus, DatabaseRestoreResult,
    ProjectDatabaseProfile,
};
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project::Project;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::secure_storage::SecureStorage;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

use super::database_cli::{
    create_database_resources, run_mysql_backup, run_mysql_restore, run_mysql_script,
    run_postgres_backup, run_postgres_restore, run_postgres_script, ProvisioningAttemptError,
    MIGRATION_TIMEOUT,
};
use super::database_identifiers::sanitize_migration_name;
use super::database_identifiers::{
    admin_url, database_display_name, database_name, default_host, generate_password, secret_key,
    username, DATABASE_SECRET_NAMESPACE,
};
use super::database_paths::{
    backup_path, collect_migration_files, create_project_paths, validate_existing_directory,
    validate_sql_file,
};

#[derive(Clone)]
pub struct LocalDatabaseProvisioner {
    storage_root: PathBuf,
    secure_storage: Arc<dyn SecureStorage>,
}

impl LocalDatabaseProvisioner {
    pub fn new(secure_storage: Arc<dyn SecureStorage>) -> AppResult<Self> {
        let project_dirs = ProjectDirs::from("dev", "AxiomPHP", "AxiomPHP").ok_or_else(|| {
            AppError::Configuration("failed to resolve application data directory".to_string())
        })?;

        Ok(Self::with_storage_root(
            project_dirs.data_local_dir().join("databases"),
            secure_storage,
        ))
    }

    pub fn with_storage_root(
        storage_root: PathBuf,
        secure_storage: Arc<dyn SecureStorage>,
    ) -> Self {
        Self {
            storage_root,
            secure_storage,
        }
    }

    fn password_for_profile(&self, profile: &ProjectDatabaseProfile) -> AppResult<String> {
        self.secure_storage
            .get_secret(DATABASE_SECRET_NAMESPACE, &secret_key(profile))?
            .ok_or_else(|| {
                AppError::Configuration(
                    "database credential was not found in secure storage".to_string(),
                )
            })
    }
}

impl DatabaseProvisioner for LocalDatabaseProvisioner {
    fn provision_project_database(
        &self,
        project: &Project,
        database_type: DatabaseType,
    ) -> AppResult<DatabaseProvisioningResult> {
        let paths = create_project_paths(&self.storage_root, &project.id, database_type)?;
        let database_name_value = database_name(&project.id, database_type);
        let password = generate_password();
        let now = Utc::now();
        let mut profile = ProjectDatabaseProfile {
            project_id: project.id.clone(),
            database_type,
            database_name: database_name_value.clone(),
            username: username(&project.id),
            host: default_host(database_type).to_string(),
            port: database_type.default_port(),
            data_dir: paths.data_dir.to_string_lossy().into_owned(),
            backup_dir: paths.backup_dir.to_string_lossy().into_owned(),
            migration_dir: paths.migration_dir.to_string_lossy().into_owned(),
            admin_url: admin_url(database_type, &database_name_value),
            status: DatabaseProvisioningStatus::Pending,
            status_message: "Database data directory and secure credential were created. Service provisioning is pending database CLI/admin access.".to_string(),
            applied_migrations: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        self.secure_storage.store_secret(
            DATABASE_SECRET_NAMESPACE,
            &secret_key(&profile),
            &password,
        )?;

        match create_database_resources(&profile, &password) {
            Ok(()) => {
                profile.status = DatabaseProvisioningStatus::Ready;
                profile.status_message = format!(
                    "{} database and user were provisioned successfully.",
                    database_display_name(database_type)
                );
                profile.updated_at = Utc::now();
                Ok(DatabaseProvisioningResult {
                    credential_stored: true,
                    database_created: true,
                    status_message: profile.status_message.clone(),
                    profile,
                })
            }
            Err(ProvisioningAttemptError::Pending(message)) => {
                profile.status = DatabaseProvisioningStatus::Pending;
                profile.status_message = message.clone();
                profile.updated_at = Utc::now();
                Ok(DatabaseProvisioningResult {
                    credential_stored: true,
                    database_created: false,
                    status_message: message,
                    profile,
                })
            }
            Err(ProvisioningAttemptError::Failed(message)) => {
                profile.status = DatabaseProvisioningStatus::Failed;
                profile.status_message = message.clone();
                profile.updated_at = Utc::now();
                Ok(DatabaseProvisioningResult {
                    credential_stored: true,
                    database_created: false,
                    status_message: message,
                    profile,
                })
            }
        }
    }

    fn backup_project_database(
        &self,
        profile: &ProjectDatabaseProfile,
    ) -> AppResult<DatabaseBackupResult> {
        ensure_profile_ready(profile)?;
        let password = self.password_for_profile(profile)?;
        let backup_path = backup_path(profile)?;

        match profile.database_type {
            DatabaseType::Mysql => run_mysql_backup(profile, &password, &backup_path)?,
            DatabaseType::Postgresql => run_postgres_backup(profile, &password, &backup_path)?,
        };

        Ok(DatabaseBackupResult {
            project_id: profile.project_id.clone(),
            database_type: profile.database_type,
            backup_path: backup_path.to_string_lossy().into_owned(),
            status_message: "Database backup completed successfully.".to_string(),
        })
    }

    fn restore_project_database(
        &self,
        profile: &ProjectDatabaseProfile,
        backup_path: &str,
    ) -> AppResult<DatabaseRestoreResult> {
        ensure_profile_ready(profile)?;
        let sql_path = validate_sql_file(backup_path)?;
        let password = self.password_for_profile(profile)?;

        match profile.database_type {
            DatabaseType::Mysql => run_mysql_restore(profile, &password, &sql_path)?,
            DatabaseType::Postgresql => run_postgres_restore(profile, &password, &sql_path)?,
        };

        Ok(DatabaseRestoreResult {
            project_id: profile.project_id.clone(),
            database_type: profile.database_type,
            backup_path: sql_path.to_string_lossy().into_owned(),
            status_message: "Database restore completed successfully.".to_string(),
        })
    }

    fn create_migration_file(
        &self,
        project_id: &ProjectId,
        database_type: DatabaseType,
        migration_dir: &str,
        name: &str,
    ) -> AppResult<DatabaseMigrationFile> {
        let migration_dir = validate_existing_directory(migration_dir, "migration directory")?;
        let migration_name = sanitize_migration_name(name)?;
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let migration_path = migration_dir.join(format!("{timestamp}_{migration_name}.sql"));
        let header = format!(
            "-- AxiomPHP migration\n-- Project: {}\n-- Database: {}\n\n",
            project_id.0,
            database_type.as_key()
        );

        fs::write(&migration_path, header).map_err(|error| {
            AppError::Infrastructure(format!("failed to create migration file: {error}"))
        })?;

        Ok(DatabaseMigrationFile {
            project_id: project_id.clone(),
            database_type,
            migration_path: migration_path.to_string_lossy().into_owned(),
            status_message: "Migration file created.".to_string(),
        })
    }

    fn run_migrations(
        &self,
        profile: &ProjectDatabaseProfile,
    ) -> AppResult<DatabaseMigrationRunResult> {
        ensure_profile_ready(profile)?;
        let migration_dir =
            validate_existing_directory(&profile.migration_dir, "migration directory")?;
        let password = self.password_for_profile(profile)?;
        let mut migration_paths = collect_migration_files(&migration_dir)?;
        migration_paths.retain(|path| {
            path.file_name()
                .and_then(|file_name| file_name.to_str())
                .is_some_and(|file_name| {
                    !profile
                        .applied_migrations
                        .iter()
                        .any(|applied| applied == file_name)
                })
        });

        let mut applied_migrations = Vec::new();
        for migration_path in migration_paths {
            match profile.database_type {
                DatabaseType::Mysql => {
                    run_mysql_script(profile, &password, &migration_path, MIGRATION_TIMEOUT)?;
                }
                DatabaseType::Postgresql => {
                    run_postgres_script(profile, &password, &migration_path, MIGRATION_TIMEOUT)?;
                }
            }

            if let Some(file_name) = migration_path.file_name().and_then(|value| value.to_str()) {
                applied_migrations.push(file_name.to_string());
            }
        }

        let status_message = if applied_migrations.is_empty() {
            "No pending database migrations found.".to_string()
        } else {
            format!(
                "Applied {} database migration(s).",
                applied_migrations.len()
            )
        };

        Ok(DatabaseMigrationRunResult {
            project_id: profile.project_id.clone(),
            database_type: profile.database_type,
            applied_migrations,
            status_message,
        })
    }
}

fn ensure_profile_ready(profile: &ProjectDatabaseProfile) -> AppResult<()> {
    if profile.status != DatabaseProvisioningStatus::Ready {
        return Err(AppError::Validation(
            "database profile is not ready for backup, restore, or migrations".to_string(),
        ));
    }

    Ok(())
}
