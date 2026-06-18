#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlCenterStatus {
    BlockedSafely,
    Checking,
    Error,
    Missing,
    NeedsSetup,
    Ready,
    Running,
    Stopped,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ControlCenterSeverity {
    Error,
    Info,
    Warning,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupDiagnostic {
    pub id: String,
    pub title: String,
    pub status: ControlCenterStatus,
    pub severity: ControlCenterSeverity,
    pub message: String,
    pub next_step: String,
    pub details: Option<String>,
}

impl SetupDiagnostic {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        status: ControlCenterStatus,
        severity: ControlCenterSeverity,
        message: impl Into<String>,
        next_step: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            status,
            severity,
            message: message.into(),
            next_step: next_step.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupDiagnosticStep {
    pub id: String,
    pub title: String,
    pub status: ControlCenterStatus,
    pub diagnostics: Vec<SetupDiagnostic>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupDiagnosticsReport {
    pub steps: Vec<SetupDiagnosticStep>,
    pub ready: bool,
    pub status_message: String,
}
