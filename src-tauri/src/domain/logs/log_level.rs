#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Debug,
    Error,
    Info,
    Warn,
}
