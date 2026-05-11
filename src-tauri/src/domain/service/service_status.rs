#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ServiceStatus {
    Unknown,
    Stopped,
    Running,
    Failed,
}
