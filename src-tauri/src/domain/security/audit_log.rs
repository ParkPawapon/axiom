use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuditSeverity {
    Error,
    Info,
    Warning,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub operation: String,
    pub resource: String,
    pub severity: AuditSeverity,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogReadResult {
    pub entries: Vec<AuditLogEntry>,
    pub returned_entries: usize,
    pub retention_days: u16,
    pub log_file: String,
    pub truncated: bool,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogRetentionResult {
    pub removed_entries: usize,
    pub retained_entries: usize,
    pub retention_days: u16,
    pub log_file: String,
    pub status_message: String,
}
