#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    ConfigurationError,
    InfrastructureError,
    NotFound,
    PermissionDenied,
    Unexpected,
    ValidationFailed,
}
