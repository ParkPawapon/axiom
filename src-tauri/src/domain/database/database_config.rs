use chrono::{DateTime, Utc};

use crate::domain::project::project_id::ProjectId;

use super::database_type::DatabaseType;

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseProvisioningStatus {
    Failed,
    Pending,
    Ready,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDatabaseProfile {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub database_name: String,
    pub username: String,
    pub host: String,
    pub port: u16,
    pub data_dir: String,
    pub backup_dir: String,
    pub migration_dir: String,
    pub admin_url: Option<String>,
    pub status: DatabaseProvisioningStatus,
    pub status_message: String,
    pub applied_migrations: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ManagedDatabaseDependencyStatus {
    Installed,
    Pending,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedDatabasePackage {
    pub package_name: String,
    pub already_installed: bool,
    pub installed_now: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedDatabaseDependencyReport {
    pub database_type: DatabaseType,
    pub provider: String,
    pub status: ManagedDatabaseDependencyStatus,
    pub packages: Vec<ManagedDatabasePackage>,
    pub diagnostics: Vec<String>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedDatabaseServiceReport {
    pub service_id: String,
    pub started: bool,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpMyAdminAccess {
    pub url: String,
    pub document_root: String,
    pub config_path: String,
    pub reverse_proxy_config_path: String,
    pub reverse_proxy_started: bool,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseProvisioningResult {
    pub profile: ProjectDatabaseProfile,
    pub credential_stored: bool,
    pub database_created: bool,
    pub dependency_report: Option<ManagedDatabaseDependencyReport>,
    pub phpmyadmin_access: Option<PhpMyAdminAccess>,
    pub service_report: Option<ManagedDatabaseServiceReport>,
    pub status_message: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseBackupCompression {
    Gzip,
    None,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseBackupEncryption {
    Aes256Gcm,
    None,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupOptions {
    pub compression: DatabaseBackupCompression,
    pub encryption: DatabaseBackupEncryption,
    pub retention_days: u16,
}

impl Default for DatabaseBackupOptions {
    fn default() -> Self {
        Self {
            compression: DatabaseBackupCompression::Gzip,
            encryption: DatabaseBackupEncryption::Aes256Gcm,
            retention_days: 30,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupMetadata {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub backup_path: String,
    pub metadata_path: String,
    pub compression: DatabaseBackupCompression,
    pub encryption: DatabaseBackupEncryption,
    pub compressed: bool,
    pub encrypted: bool,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupResult {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub backup_path: String,
    pub metadata_path: Option<String>,
    pub compression: DatabaseBackupCompression,
    pub encryption: DatabaseBackupEncryption,
    pub compressed: bool,
    pub encrypted: bool,
    pub size_bytes: u64,
    pub pruned_backup_paths: Vec<String>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRestoreResult {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub backup_path: String,
    pub restored_from_path: String,
    pub decrypted: bool,
    pub decompressed: bool,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupPolicy {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub enabled: bool,
    pub interval_minutes: u32,
    pub retention_days: u16,
    pub compression: DatabaseBackupCompression,
    pub encryption: DatabaseBackupEncryption,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl DatabaseBackupPolicy {
    pub fn backup_options(&self) -> DatabaseBackupOptions {
        DatabaseBackupOptions {
            compression: self.compression,
            encryption: self.encryption,
            retention_days: self.retention_days,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupPolicyUpdate {
    pub enabled: bool,
    pub interval_minutes: u32,
    pub retention_days: u16,
    pub compression: DatabaseBackupCompression,
    pub encryption: DatabaseBackupEncryption,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupPolicyUpdateResult {
    pub policy: DatabaseBackupPolicy,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledDatabaseBackupRunResult {
    pub checked_policies: usize,
    pub completed_backups: usize,
    pub skipped_backups: usize,
    pub backups: Vec<DatabaseBackupResult>,
    pub errors: Vec<String>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseMigrationFile {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub migration_path: String,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseMigrationRunResult {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub applied_migrations: Vec<String>,
    pub status_message: String,
}
