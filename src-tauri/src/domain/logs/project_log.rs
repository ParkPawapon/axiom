use super::log_entry::LogEntry;
use crate::domain::project::project_id::ProjectId;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectLogReadResult {
    pub project_id: ProjectId,
    pub log_file: String,
    pub entries: Vec<LogEntry>,
    pub returned_lines: usize,
    pub file_size_bytes: u64,
    pub truncated: bool,
    pub status_message: String,
}
