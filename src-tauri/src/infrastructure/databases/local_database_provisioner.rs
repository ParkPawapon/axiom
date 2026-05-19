use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::Utc;
use directories::ProjectDirs;
use sha2::{Digest, Sha256};

use crate::domain::database::database_config::{
    DatabaseBackupOptions, DatabaseBackupResult, DatabaseContinuousReplayRestoreResult,
    DatabaseMigrationFile, DatabaseMigrationRollbackGenerationResult,
    DatabaseMigrationRollbackResult, DatabaseMigrationRunResult, DatabaseProvisioningResult,
    DatabaseProvisioningStatus, DatabaseReplaySegment, DatabaseReplaySegmentKind,
    DatabaseRestoreResult, ProjectDatabaseProfile,
};
use crate::domain::database::database_type::DatabaseType;
use crate::domain::project::project::Project;
use crate::domain::project::project_id::ProjectId;
use crate::ports::database_provisioner::DatabaseProvisioner;
use crate::ports::secure_storage::SecureStorage;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

use super::backup_artifacts::{
    cleanup_restore_artifact, finalize_backup_artifact, prepare_restore_artifact,
};
use super::database_cli::{
    create_database_resources, run_mysql_backup, run_mysql_binlog_export, run_mysql_restore,
    run_mysql_script, run_postgres_backup, run_postgres_restore, run_postgres_script,
    ProvisioningAttemptError, MIGRATION_TIMEOUT,
};
use super::database_identifiers::sanitize_migration_name;
use super::database_identifiers::{
    admin_url, database_display_name, database_name, default_host, generate_password, secret_key,
    username, DATABASE_SECRET_NAMESPACE,
};
use super::database_paths::{
    backup_path, collect_migration_files, create_project_paths, validate_existing_directory,
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
                    dependency_report: None,
                    phpmyadmin_access: None,
                    status_message: profile.status_message.clone(),
                    service_report: None,
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
                    dependency_report: None,
                    phpmyadmin_access: None,
                    status_message: message,
                    service_report: None,
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
                    dependency_report: None,
                    phpmyadmin_access: None,
                    status_message: message,
                    service_report: None,
                    profile,
                })
            }
        }
    }

    fn backup_project_database(
        &self,
        profile: &ProjectDatabaseProfile,
        options: DatabaseBackupOptions,
    ) -> AppResult<DatabaseBackupResult> {
        ensure_profile_ready(profile)?;
        let password = self.password_for_profile(profile)?;
        let backup_path = backup_path(profile)?;

        match profile.database_type {
            DatabaseType::Mysql => run_mysql_backup(profile, &password, &backup_path)?,
            DatabaseType::Postgresql => run_postgres_backup(profile, &password, &backup_path)?,
        };
        let artifact =
            finalize_backup_artifact(profile, self.secure_storage.as_ref(), backup_path, options)?;

        Ok(DatabaseBackupResult {
            project_id: profile.project_id.clone(),
            database_type: profile.database_type,
            backup_path: artifact.backup_path.to_string_lossy().into_owned(),
            metadata_path: Some(artifact.metadata_path.to_string_lossy().into_owned()),
            signature_path: Some(artifact.signature_path.to_string_lossy().into_owned()),
            compression: options.compression,
            encryption: options.encryption,
            compressed: artifact.compressed,
            encrypted: artifact.encrypted,
            size_bytes: artifact.size_bytes,
            pruned_backup_paths: artifact.pruned_backup_paths,
            remote_copy_paths: Vec::new(),
            remote_copy_receipts: Vec::new(),
            status_message:
                "Database backup completed with managed retention and artifact processing."
                    .to_string(),
        })
    }

    fn restore_project_database(
        &self,
        profile: &ProjectDatabaseProfile,
        backup_path: &str,
    ) -> AppResult<DatabaseRestoreResult> {
        ensure_profile_ready(profile)?;
        let restore_artifact =
            prepare_restore_artifact(profile, self.secure_storage.as_ref(), backup_path)?;
        let password = self.password_for_profile(profile)?;

        let restore_result = match profile.database_type {
            DatabaseType::Mysql => {
                run_mysql_restore(profile, &password, &restore_artifact.sql_path)
            }
            DatabaseType::Postgresql => {
                run_postgres_restore(profile, &password, &restore_artifact.sql_path)
            }
        };

        let restore_result = match restore_result {
            Ok(output) => output,
            Err(error) => {
                cleanup_restore_artifact(&restore_artifact);
                return Err(error);
            }
        };
        cleanup_restore_artifact(&restore_artifact);
        let status_message = if restore_result.stderr.trim().is_empty() {
            "Database restore completed successfully.".to_string()
        } else {
            "Database restore completed successfully with command diagnostics.".to_string()
        };

        Ok(DatabaseRestoreResult {
            project_id: profile.project_id.clone(),
            database_type: profile.database_type,
            backup_path: restore_artifact.sql_path.to_string_lossy().into_owned(),
            restored_from_path: restore_artifact.source_path.to_string_lossy().into_owned(),
            decrypted: restore_artifact.decrypted,
            decompressed: restore_artifact.decompressed,
            signature_verified: restore_artifact.signature_verified,
            status_message,
        })
    }

    fn restore_project_database_with_replay(
        &self,
        profile: &ProjectDatabaseProfile,
        base_backup_path: &str,
        replay_source_path: &str,
        target_time: Option<chrono::DateTime<Utc>>,
    ) -> AppResult<DatabaseContinuousReplayRestoreResult> {
        ensure_profile_ready(profile)?;
        let restore = self.restore_project_database(profile, base_backup_path)?;
        let replay_log_paths =
            collect_replay_log_files(replay_source_path, profile.database_type, target_time)?;
        let password = self.password_for_profile(profile)?;
        let replay_work_dir = replay_work_dir(profile)?;
        fs::create_dir_all(&replay_work_dir).map_err(|error| {
            AppError::Infrastructure(format!("failed to create replay work directory: {error}"))
        })?;

        let mut replayed_log_paths = Vec::new();
        let mut replay_segments = Vec::new();
        for replay_log_path in replay_log_paths {
            let sql_path = prepare_replay_sql(profile, &replay_log_path, &replay_work_dir)?;

            match profile.database_type {
                DatabaseType::Mysql => {
                    run_mysql_script(profile, &password, &sql_path, MIGRATION_TIMEOUT)?;
                }
                DatabaseType::Postgresql => {
                    run_postgres_script(profile, &password, &sql_path, MIGRATION_TIMEOUT)?;
                }
            }

            replay_segments.push(DatabaseReplaySegment {
                kind: replay_segment_kind(profile.database_type, &replay_log_path)?,
                source_path: replay_log_path.to_string_lossy().into_owned(),
                applied_sql_path: sql_path.to_string_lossy().into_owned(),
                sha256: sha256_file_hex(&replay_log_path)?,
                applied_at: Utc::now(),
            });
            if sql_path.starts_with(&replay_work_dir) {
                let _ = fs::remove_file(&sql_path);
            }
            replayed_log_paths.push(replay_log_path.to_string_lossy().into_owned());
        }

        Ok(DatabaseContinuousReplayRestoreResult {
            project_id: profile.project_id.clone(),
            database_type: profile.database_type,
            base_backup_path: base_backup_path.to_string(),
            replay_source_path: replay_source_path.to_string(),
            target_time,
            restore,
            status_message: format!(
                "Database restore completed and replayed {} recovery log segment(s).",
                replayed_log_paths.len()
            ),
            replayed_log_paths,
            replay_segments,
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
        let rollback_path = migration_dir.join(format!("{timestamp}_{migration_name}.down.sql"));
        let header = format!(
            "-- AxiomPHP migration\n-- Project: {}\n-- Database: {}\n\n",
            project_id.0,
            database_type.as_key()
        );
        let rollback_header = format!(
            "-- AxiomPHP migration rollback\n-- Project: {}\n-- Database: {}\n\n",
            project_id.0,
            database_type.as_key()
        );

        fs::write(&migration_path, header).map_err(|error| {
            AppError::Infrastructure(format!("failed to create migration file: {error}"))
        })?;
        if let Err(error) = fs::write(&rollback_path, rollback_header) {
            let _ = fs::remove_file(&migration_path);
            return Err(AppError::Infrastructure(format!(
                "failed to create migration rollback file: {error}"
            )));
        }

        Ok(DatabaseMigrationFile {
            project_id: project_id.clone(),
            database_type,
            migration_path: migration_path.to_string_lossy().into_owned(),
            rollback_path: Some(rollback_path.to_string_lossy().into_owned()),
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

    fn rollback_migrations(
        &self,
        profile: &ProjectDatabaseProfile,
        steps: u16,
    ) -> AppResult<DatabaseMigrationRollbackResult> {
        ensure_profile_ready(profile)?;
        let migration_dir =
            validate_existing_directory(&profile.migration_dir, "migration directory")?;
        let password = self.password_for_profile(profile)?;
        let rollback_candidates = profile
            .applied_migrations
            .iter()
            .rev()
            .take(usize::from(steps))
            .cloned()
            .collect::<Vec<_>>();

        if rollback_candidates.is_empty() {
            return Ok(DatabaseMigrationRollbackResult {
                project_id: profile.project_id.clone(),
                database_type: profile.database_type,
                rolled_back_migrations: Vec::new(),
                status_message: "No applied database migrations were available to roll back."
                    .to_string(),
            });
        }

        let mut rolled_back_migrations = Vec::new();
        for migration_name in rollback_candidates {
            let rollback_path = rollback_path_for(&migration_dir, &migration_name)?;

            match profile.database_type {
                DatabaseType::Mysql => {
                    run_mysql_script(profile, &password, &rollback_path, MIGRATION_TIMEOUT)?;
                }
                DatabaseType::Postgresql => {
                    run_postgres_script(profile, &password, &rollback_path, MIGRATION_TIMEOUT)?;
                }
            }

            rolled_back_migrations.push(migration_name);
        }

        Ok(DatabaseMigrationRollbackResult {
            project_id: profile.project_id.clone(),
            database_type: profile.database_type,
            status_message: format!(
                "Rolled back {} database migration(s).",
                rolled_back_migrations.len()
            ),
            rolled_back_migrations,
        })
    }

    fn generate_migration_rollback(
        &self,
        profile: &ProjectDatabaseProfile,
        migration_path: &str,
    ) -> AppResult<DatabaseMigrationRollbackGenerationResult> {
        let migration_path = validate_migration_path(profile, migration_path)?;
        let rollback_path = rollback_path_for_generated_migration(&migration_path)?;
        let migration_sql = fs::read_to_string(&migration_path).map_err(|error| {
            AppError::Infrastructure(format!("failed to read migration SQL: {error}"))
        })?;
        let generated = generate_rollback_sql(&migration_sql);

        fs::write(&rollback_path, generated.contents()).map_err(|error| {
            AppError::Infrastructure(format!("failed to write generated rollback SQL: {error}"))
        })?;

        Ok(DatabaseMigrationRollbackGenerationResult {
            project_id: profile.project_id.clone(),
            database_type: profile.database_type,
            migration_path: migration_path.to_string_lossy().into_owned(),
            rollback_path: rollback_path.to_string_lossy().into_owned(),
            generated_statements: generated.statements,
            warnings: generated.warnings,
            status_message: "Rollback SQL was generated from reversible migration patterns."
                .to_string(),
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

fn rollback_path_for(migration_dir: &std::path::Path, migration_name: &str) -> AppResult<PathBuf> {
    if migration_name.contains(std::path::MAIN_SEPARATOR) || migration_name.ends_with(".down.sql") {
        return Err(AppError::Validation(
            "applied migration name is not valid for rollback".to_string(),
        ));
    }

    let Some(stem) = migration_name.strip_suffix(".sql") else {
        return Err(AppError::Validation(
            "applied migration must be a .sql migration file".to_string(),
        ));
    };
    let rollback_path = migration_dir.join(format!("{stem}.down.sql"));

    if !rollback_path.is_file() {
        return Err(AppError::Validation(format!(
            "rollback migration file is missing for `{migration_name}`"
        )));
    }

    Ok(rollback_path)
}

fn replay_work_dir(profile: &ProjectDatabaseProfile) -> AppResult<PathBuf> {
    let backup_dir = std::path::Path::new(&profile.backup_dir)
        .canonicalize()
        .map_err(|error| {
            AppError::Validation(format!(
                "backup directory must exist and be readable: {error}"
            ))
        })?;

    Ok(backup_dir.join(".replay-work"))
}

fn collect_replay_log_files(
    replay_source_path: &str,
    database_type: DatabaseType,
    target_time: Option<chrono::DateTime<Utc>>,
) -> AppResult<Vec<PathBuf>> {
    let replay_source_path =
        validate_existing_directory(replay_source_path, "recovery replay source directory")?;
    let mut replay_paths = Vec::new();

    for entry in fs::read_dir(&replay_source_path).map_err(|error| {
        AppError::Infrastructure(format!("failed to read recovery replay directory: {error}"))
    })? {
        let path = entry
            .map_err(|error| {
                AppError::Infrastructure(format!("failed to inspect recovery replay file: {error}"))
            })?
            .path();

        if !path.is_file() || !is_replay_file(database_type, &path)? {
            continue;
        }

        if let Some(target_time) = target_time {
            let modified_at = path
                .metadata()
                .and_then(|metadata| metadata.modified())
                .map_err(|error| {
                    AppError::Infrastructure(format!(
                        "failed to inspect recovery replay file metadata: {error}"
                    ))
                })?;
            let modified_at = chrono::DateTime::<Utc>::from(modified_at);

            if modified_at > target_time {
                continue;
            }
        }

        replay_paths.push(path);
    }

    replay_paths.sort();
    Ok(replay_paths)
}

fn is_replay_file(database_type: DatabaseType, path: &std::path::Path) -> AppResult<bool> {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if file_name.ends_with(".sql") {
        return Ok(true);
    }

    match database_type {
        DatabaseType::Mysql => Ok(is_mysql_binlog_name(&file_name)),
        DatabaseType::Postgresql if file_name.ends_with(".wal") => Err(AppError::Validation(
            "PostgreSQL WAL physical replay requires server-level restore orchestration; provide WAL-derived .sql replay segments for this managed flow".to_string(),
        )),
        DatabaseType::Postgresql => Ok(false),
    }
}

fn replay_segment_kind(
    database_type: DatabaseType,
    path: &std::path::Path,
) -> AppResult<DatabaseReplaySegmentKind> {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if database_type == DatabaseType::Mysql && is_mysql_binlog_name(&file_name) {
        return Ok(DatabaseReplaySegmentKind::MysqlBinlog);
    }

    if database_type == DatabaseType::Postgresql && file_name.ends_with(".wal.sql") {
        return Ok(DatabaseReplaySegmentKind::PostgresWalSql);
    }

    if file_name.ends_with(".sql") {
        return Ok(DatabaseReplaySegmentKind::Sql);
    }

    Err(AppError::Validation(
        "replay segment type is not supported".to_string(),
    ))
}

fn is_mysql_binlog_name(file_name: &str) -> bool {
    file_name.ends_with(".binlog")
        || file_name.ends_with(".bin")
        || file_name.starts_with("mysql-bin.")
        || file_name.starts_with("mariadb-bin.")
}

fn prepare_replay_sql(
    profile: &ProjectDatabaseProfile,
    replay_log_path: &std::path::Path,
    replay_work_dir: &std::path::Path,
) -> AppResult<PathBuf> {
    let file_name = replay_log_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if file_name.ends_with(".sql") {
        return Ok(replay_log_path.to_path_buf());
    }

    if profile.database_type != DatabaseType::Mysql {
        return Err(AppError::Validation(
            "only MySQL binlog files can be converted by the managed replay adapter".to_string(),
        ));
    }

    let output_path = replay_work_dir.join(format!(
        "{}.replay.sql",
        replay_log_path
            .file_stem()
            .and_then(|file_stem| file_stem.to_str())
            .unwrap_or("mysql-binlog")
    ));
    run_mysql_binlog_export(replay_log_path, &output_path)?;

    Ok(output_path)
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct GeneratedRollbackSql {
    statements: Vec<String>,
    warnings: Vec<String>,
}

impl GeneratedRollbackSql {
    fn contents(&self) -> String {
        let mut contents =
            "-- AxiomPHP generated migration rollback\n-- Review before running.\n\n".to_string();

        for warning in &self.warnings {
            contents.push_str("-- WARNING: ");
            contents.push_str(warning);
            contents.push('\n');
        }

        if !self.warnings.is_empty() {
            contents.push('\n');
        }

        for statement in &self.statements {
            contents.push_str(statement);
            contents.push_str("\n");
        }

        contents
    }
}

fn validate_migration_path(
    profile: &ProjectDatabaseProfile,
    migration_path: &str,
) -> AppResult<PathBuf> {
    let migration_dir = validate_existing_directory(&profile.migration_dir, "migration directory")?;
    let migration_path = std::path::Path::new(migration_path.trim())
        .canonicalize()
        .map_err(|error| {
            AppError::Validation(format!("migration path is not readable: {error}"))
        })?;

    if !migration_path.starts_with(&migration_dir) {
        return Err(AppError::Validation(
            "migration path must be inside the profile migration directory".to_string(),
        ));
    }

    let file_name = migration_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();

    if !file_name.ends_with(".sql") || file_name.ends_with(".down.sql") {
        return Err(AppError::Validation(
            "migration path must point to a forward .sql file".to_string(),
        ));
    }

    Ok(migration_path)
}

fn rollback_path_for_generated_migration(migration_path: &std::path::Path) -> AppResult<PathBuf> {
    let file_name = migration_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .ok_or_else(|| AppError::Validation("migration path has no file name".to_string()))?;
    let stem = file_name.strip_suffix(".sql").ok_or_else(|| {
        AppError::Validation("migration path must point to a .sql file".to_string())
    })?;

    Ok(migration_path.with_file_name(format!("{stem}.down.sql")))
}

fn generate_rollback_sql(sql: &str) -> GeneratedRollbackSql {
    let mut statements = Vec::new();
    let mut warnings = Vec::new();

    for statement in sql.split(';') {
        let statement = statement.trim();
        if statement.is_empty() || statement.starts_with("--") {
            continue;
        }

        if let Some(rollback) = rollback_create_table(statement) {
            statements.push(rollback);
        } else if let Some(rollback) = rollback_create_index(statement) {
            statements.push(rollback);
        } else if let Some(rollback) = rollback_create_view(statement) {
            statements.push(rollback);
        } else if let Some(rollback) = rollback_create_schema(statement) {
            statements.push(rollback);
        } else if let Some(rollback) = rollback_rename_table(statement) {
            statements.push(rollback);
        } else if let Some(rollback) = rollback_rename_column(statement) {
            statements.push(rollback);
        } else if let Some(rollback) = rollback_add_constraint(statement) {
            statements.push(rollback);
        } else if let Some(rollback) = rollback_add_column(statement) {
            statements.push(rollback);
        } else {
            warnings.push(format!(
                "Unsupported migration statement was not auto-reversed: {}",
                first_words(statement, 12)
            ));
        }
    }

    statements.reverse();

    GeneratedRollbackSql {
        statements,
        warnings,
    }
}

fn rollback_create_table(statement: &str) -> Option<String> {
    let tokens = statement.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 3
        || !tokens[0].eq_ignore_ascii_case("create")
        || !tokens[1].eq_ignore_ascii_case("table")
    {
        return None;
    }

    let table_name = if tokens
        .get(2)
        .is_some_and(|token| token.eq_ignore_ascii_case("if"))
    {
        tokens.get(5)?
    } else {
        tokens.get(2)?
    };

    Some(format!(
        "DROP TABLE IF EXISTS {};",
        trim_identifier_suffix(table_name)
    ))
}

fn rollback_create_index(statement: &str) -> Option<String> {
    let tokens = statement.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 3
        || !tokens[0].eq_ignore_ascii_case("create")
        || !(tokens[1].eq_ignore_ascii_case("index")
            || (tokens[1].eq_ignore_ascii_case("unique")
                && tokens
                    .get(2)
                    .is_some_and(|token| token.eq_ignore_ascii_case("index"))))
    {
        return None;
    }

    let index_name = if tokens[1].eq_ignore_ascii_case("unique") {
        tokens.get(3)?
    } else {
        tokens.get(2)?
    };

    Some(format!(
        "DROP INDEX IF EXISTS {};",
        trim_identifier_suffix(index_name)
    ))
}

fn rollback_create_view(statement: &str) -> Option<String> {
    let tokens = statement.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 3
        || !tokens[0].eq_ignore_ascii_case("create")
        || !tokens[1].eq_ignore_ascii_case("view")
    {
        return None;
    }

    Some(format!(
        "DROP VIEW IF EXISTS {};",
        trim_identifier_suffix(tokens[2])
    ))
}

fn rollback_create_schema(statement: &str) -> Option<String> {
    let tokens = statement.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 3
        || !tokens[0].eq_ignore_ascii_case("create")
        || !tokens[1].eq_ignore_ascii_case("schema")
    {
        return None;
    }

    let schema_name = if tokens
        .get(2)
        .is_some_and(|token| token.eq_ignore_ascii_case("if"))
    {
        tokens.get(5)?
    } else {
        tokens.get(2)?
    };

    Some(format!(
        "DROP SCHEMA IF EXISTS {};",
        trim_identifier_suffix(schema_name)
    ))
}

fn rollback_add_column(statement: &str) -> Option<String> {
    let tokens = statement.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 6
        || !tokens[0].eq_ignore_ascii_case("alter")
        || !tokens[1].eq_ignore_ascii_case("table")
        || !tokens[3].eq_ignore_ascii_case("add")
    {
        return None;
    }

    let column_index = if tokens
        .get(4)
        .is_some_and(|token| token.eq_ignore_ascii_case("column"))
    {
        5
    } else {
        4
    };

    Some(format!(
        "ALTER TABLE {} DROP COLUMN {};",
        trim_identifier_suffix(tokens[2]),
        trim_identifier_suffix(tokens[column_index])
    ))
}

fn rollback_add_constraint(statement: &str) -> Option<String> {
    let tokens = statement.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 7
        || !tokens[0].eq_ignore_ascii_case("alter")
        || !tokens[1].eq_ignore_ascii_case("table")
        || !tokens[3].eq_ignore_ascii_case("add")
        || !tokens
            .get(4)
            .is_some_and(|token| token.eq_ignore_ascii_case("constraint"))
    {
        return None;
    }

    Some(format!(
        "ALTER TABLE {} DROP CONSTRAINT {};",
        trim_identifier_suffix(tokens[2]),
        trim_identifier_suffix(tokens[5])
    ))
}

fn rollback_rename_table(statement: &str) -> Option<String> {
    let tokens = statement.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 6
        || !tokens[0].eq_ignore_ascii_case("alter")
        || !tokens[1].eq_ignore_ascii_case("table")
        || !tokens[3].eq_ignore_ascii_case("rename")
        || !tokens[4].eq_ignore_ascii_case("to")
    {
        return None;
    }

    Some(format!(
        "ALTER TABLE {} RENAME TO {};",
        trim_identifier_suffix(tokens[5]),
        trim_identifier_suffix(tokens[2])
    ))
}

fn rollback_rename_column(statement: &str) -> Option<String> {
    let tokens = statement.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 8
        || !tokens[0].eq_ignore_ascii_case("alter")
        || !tokens[1].eq_ignore_ascii_case("table")
        || !tokens[3].eq_ignore_ascii_case("rename")
        || !tokens[4].eq_ignore_ascii_case("column")
        || !tokens[6].eq_ignore_ascii_case("to")
    {
        return None;
    }

    Some(format!(
        "ALTER TABLE {} RENAME COLUMN {} TO {};",
        trim_identifier_suffix(tokens[2]),
        trim_identifier_suffix(tokens[7]),
        trim_identifier_suffix(tokens[5])
    ))
}

fn trim_identifier_suffix(value: &str) -> &str {
    value.trim_end_matches(|character| matches!(character, '(' | ',' | ';'))
}

fn first_words(value: &str, limit: usize) -> String {
    value
        .split_whitespace()
        .take(limit)
        .collect::<Vec<_>>()
        .join(" ")
}

fn sha256_file_hex(path: &std::path::Path) -> AppResult<String> {
    let mut file = File::open(path).map_err(|error| {
        AppError::Infrastructure(format!(
            "failed to open replay segment for hashing: {error}"
        ))
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read_count = file.read(&mut buffer).map_err(|error| {
            AppError::Infrastructure(format!("failed to hash replay segment: {error}"))
        })?;

        if read_count == 0 {
            break;
        }

        hasher.update(&buffer[..read_count]);
    }

    Ok(hex_encode(&hasher.finalize()))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }

    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_conservative_rollbacks_for_common_reversible_patterns() {
        let generated = generate_rollback_sql(
            r#"
            CREATE TABLE users (id int);
            ALTER TABLE users ADD COLUMN email text;
            CREATE INDEX users_email_idx ON users(email);
            ALTER TABLE users ADD CONSTRAINT users_email_unique UNIQUE(email);
            CREATE VIEW active_users AS SELECT * FROM users;
            ALTER TABLE users RENAME COLUMN email TO email_address;
            "#,
        );

        assert!(generated
            .statements
            .iter()
            .any(|statement| statement == "DROP TABLE IF EXISTS users;"));
        assert!(generated
            .statements
            .iter()
            .any(|statement| statement == "ALTER TABLE users DROP COLUMN email;"));
        assert!(generated
            .statements
            .iter()
            .any(|statement| statement == "DROP INDEX IF EXISTS users_email_idx;"));
        assert!(generated.statements.iter().any(|statement| {
            statement == "ALTER TABLE users DROP CONSTRAINT users_email_unique;"
        }));
        assert!(generated
            .statements
            .iter()
            .any(|statement| statement == "DROP VIEW IF EXISTS active_users;"));
        assert!(generated.statements.iter().any(|statement| {
            statement == "ALTER TABLE users RENAME COLUMN email_address TO email;"
        }));
    }

    #[test]
    fn classifies_replay_segments() {
        assert_eq!(
            replay_segment_kind(
                DatabaseType::Mysql,
                std::path::Path::new("mysql-bin.000001")
            )
            .expect("mysql binlog"),
            DatabaseReplaySegmentKind::MysqlBinlog
        );
        assert_eq!(
            replay_segment_kind(
                DatabaseType::Postgresql,
                std::path::Path::new("0001.wal.sql")
            )
            .expect("postgres wal sql"),
            DatabaseReplaySegmentKind::PostgresWalSql
        );
    }
}
