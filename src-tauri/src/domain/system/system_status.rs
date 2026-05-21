use chrono::{DateTime, Utc};

use crate::domain::docker::docker_project::DockerDiagnosticsReport;
use crate::domain::security::security_status::SecurityPermissionStatus;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PortCheckState {
    Available,
    InUse,
    Unavailable,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortCheckResult {
    pub port: u16,
    pub bind_address: String,
    pub state: PortCheckState,
    pub checked_at: DateTime<Utc>,
    pub status_message: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerCheckResult {
    pub diagnostics: DockerDiagnosticsReport,
    pub checked_at: DateTime<Utc>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionCheckResult {
    pub permissions: SecurityPermissionStatus,
    pub checked_at: DateTime<Utc>,
    pub status_message: String,
}
