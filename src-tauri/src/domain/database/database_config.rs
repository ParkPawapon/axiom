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
    #[serde(default)]
    pub kms_envelope: Option<DatabaseBackupKmsEnvelope>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupKmsEnvelope {
    pub provider: String,
    pub key_id: String,
    pub encrypted_data_key_fingerprint: String,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupRemoteCopyReceipt {
    pub provider: DatabaseBackupRemoteDestinationProvider,
    pub artifact_path: String,
    pub remote_uri: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub copied_at: DateTime<Utc>,
    pub verified: bool,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupResult {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub backup_path: String,
    pub metadata_path: Option<String>,
    pub signature_path: Option<String>,
    pub compression: DatabaseBackupCompression,
    pub encryption: DatabaseBackupEncryption,
    pub compressed: bool,
    pub encrypted: bool,
    pub size_bytes: u64,
    pub pruned_backup_paths: Vec<String>,
    pub remote_copy_paths: Vec<String>,
    #[serde(default)]
    pub remote_copy_receipts: Vec<DatabaseBackupRemoteCopyReceipt>,
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
    pub signature_verified: bool,
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
pub struct DatabaseBackupRemoteDestination {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    #[serde(default)]
    pub provider: DatabaseBackupRemoteDestinationProvider,
    pub enabled: bool,
    pub destination_path: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseBackupRemoteDestinationProvider {
    Gcs,
    #[default]
    LocalPath,
    R2,
    S3,
    Sftp,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupRemoteDestinationUpdate {
    #[serde(default)]
    pub provider: DatabaseBackupRemoteDestinationProvider,
    pub enabled: bool,
    pub destination_path: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupRemoteDestinationUpdateResult {
    pub destination: DatabaseBackupRemoteDestination,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupSchedulerStatus {
    pub installed: bool,
    pub platform: String,
    pub schedule_label: String,
    pub manifest_path: Option<String>,
    pub last_checked_at: DateTime<Utc>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupSchedulerInstallResult {
    pub status: DatabaseBackupSchedulerStatus,
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
    pub rollback_path: Option<String>,
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

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseMigrationRollbackResult {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub rolled_back_migrations: Vec<String>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseMigrationRollbackGenerationResult {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub migration_path: String,
    pub rollback_path: String,
    pub generated_statements: Vec<String>,
    pub warnings: Vec<String>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabasePointInTimeRestoreResult {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub target_time: DateTime<Utc>,
    pub selected_backup_path: String,
    pub selected_backup_created_at: DateTime<Utc>,
    pub restore: DatabaseRestoreResult,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseContinuousReplayRestoreResult {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub base_backup_path: String,
    pub replay_source_path: String,
    pub target_time: Option<DateTime<Utc>>,
    pub restore: DatabaseRestoreResult,
    pub replayed_log_paths: Vec<String>,
    #[serde(default)]
    pub replay_segments: Vec<DatabaseReplaySegment>,
    pub status_message: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseReplaySegmentKind {
    MysqlBinlog,
    PostgresWalSql,
    Sql,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseReplaySegment {
    pub kind: DatabaseReplaySegmentKind,
    pub source_path: String,
    pub applied_sql_path: String,
    pub sha256: String,
    pub applied_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupKeyManagementStatus {
    pub encryption_key_source: String,
    pub signing_key_source: String,
    pub external_kms_provider: Option<String>,
    pub external_kms_key_id: Option<String>,
    pub trusted_signing_key_fingerprints: Vec<String>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupTrustBundle {
    pub version: u16,
    pub algorithm: String,
    pub signing_key_fingerprint: String,
    #[serde(default)]
    pub artifact_sha256: Option<String>,
    #[serde(default)]
    pub source_machine: Option<String>,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupTrustExportResult {
    pub trust_bundle_path: String,
    pub signing_key_fingerprint: String,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupTrustImportResult {
    pub trust_bundle_path: String,
    pub trusted_signing_key_fingerprint: String,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupArtifactTrustEnrollmentResult {
    pub backup_path: String,
    pub artifact_sha256: String,
    pub trusted_signing_key_fingerprint: Option<String>,
    pub status_message: String,
}
