#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    ValidationFailed,
    PermissionDenied,
    ConfigurationError,
    InfrastructureError,
    Unexpected,
}
