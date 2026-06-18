use chrono::{DateTime, Utc};

use crate::domain::logs::log_entry::LogEntry;
use crate::domain::project::project::Project;

use super::quick_action::QuickActionSummary;
use super::setup_diagnostic::{
    ControlCenterSeverity, ControlCenterStatus, SetupDiagnostic, SetupDiagnosticsReport,
};

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlCenterProjectSummary {
    pub id: String,
    pub name: String,
    pub document_root: String,
    pub php_url: Option<String>,
    pub php_port: Option<u16>,
    pub php_version: Option<String>,
}

impl From<&Project> for ControlCenterProjectSummary {
    fn from(project: &Project) -> Self {
        Self {
            id: project.id.0.clone(),
            name: project.name.clone(),
            document_root: project.document_root.0.clone(),
            php_url: None,
            php_port: None,
            php_version: None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlCenterServiceSummary {
    pub id: String,
    pub label: String,
    pub status: ControlCenterStatus,
    pub primary_detail: String,
    pub secondary_detail: Option<String>,
    pub can_start: bool,
    pub can_stop: bool,
    pub can_restart: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlCenterPortSummary {
    pub id: String,
    pub label: String,
    pub port: u16,
    pub status: ControlCenterStatus,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlCenterLogPreview {
    pub source: String,
    pub entries: Vec<LogEntry>,
    pub status: ControlCenterStatus,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlCenterSummary {
    pub projects: Vec<ControlCenterProjectSummary>,
    pub selected_project: Option<ControlCenterProjectSummary>,
    pub services: Vec<ControlCenterServiceSummary>,
    pub diagnostics: Vec<SetupDiagnostic>,
    pub quick_actions: Vec<QuickActionSummary>,
    pub ports: Vec<ControlCenterPortSummary>,
    pub log_preview: ControlCenterLogPreview,
    pub setup: SetupDiagnosticsReport,
    pub generated_at: DateTime<Utc>,
    pub status: ControlCenterStatus,
    pub status_message: String,
}

impl ControlCenterSummary {
    pub fn readiness_status(diagnostics: &[SetupDiagnostic]) -> ControlCenterStatus {
        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == ControlCenterSeverity::Error)
        {
            return ControlCenterStatus::BlockedSafely;
        }

        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == ControlCenterSeverity::Warning)
        {
            return ControlCenterStatus::NeedsSetup;
        }

        ControlCenterStatus::Ready
    }
}
