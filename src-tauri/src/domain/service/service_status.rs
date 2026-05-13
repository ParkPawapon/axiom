#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceStatus {
    Detected,
    Failed,
    NotConfigured,
    Running,
    Stopped,
    Unknown,
}
