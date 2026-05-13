use chrono::{DateTime, Utc};

use crate::domain::runtime::runtime_version::RuntimeVersion;

use super::project_id::ProjectId;

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectPhpProcessState {
    Failed,
    Running,
    Stopped,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPhpProcessStatus {
    pub project_id: ProjectId,
    pub state: ProjectPhpProcessState,
    pub pid: Option<u32>,
    pub php_version: Option<RuntimeVersion>,
    pub port: Option<u16>,
    pub url: Option<String>,
    pub document_root: Option<String>,
    pub log_file: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub status_message: String,
}

impl ProjectPhpProcessStatus {
    pub fn stopped(project_id: ProjectId) -> Self {
        Self {
            project_id,
            state: ProjectPhpProcessState::Stopped,
            pid: None,
            php_version: None,
            port: None,
            url: None,
            document_root: None,
            log_file: None,
            started_at: None,
            status_message: "No PHP project process is running.".to_string(),
        }
    }
}
