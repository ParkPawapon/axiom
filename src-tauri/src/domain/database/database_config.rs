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

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseBackupResult {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub backup_path: String,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRestoreResult {
    pub project_id: ProjectId,
    pub database_type: DatabaseType,
    pub backup_path: String,
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
