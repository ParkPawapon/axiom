use super::setup_diagnostic::ControlCenterStatus;

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum QuickActionKind {
    OpenBrowser,
    OpenFolder,
    OpenLogs,
    OpenPhpMyAdmin,
    RestartProject,
    StartProject,
    StopProject,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickActionSummary {
    pub id: String,
    pub label: String,
    pub kind: QuickActionKind,
    pub status: ControlCenterStatus,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
    pub target: Option<String>,
}

impl QuickActionSummary {
    pub fn enabled(
        kind: QuickActionKind,
        id: impl Into<String>,
        label: impl Into<String>,
        target: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            status: ControlCenterStatus::Ready,
            enabled: true,
            disabled_reason: None,
            target,
        }
    }

    pub fn disabled(
        kind: QuickActionKind,
        id: impl Into<String>,
        label: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            status: ControlCenterStatus::BlockedSafely,
            enabled: false,
            disabled_reason: Some(reason.into()),
            target: None,
        }
    }
}
