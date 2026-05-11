use thiserror::Error;

use super::error_code::ErrorCode;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
    #[error("unexpected application error")]
    Unexpected,
}

impl AppError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Validation(_) => ErrorCode::ValidationFailed,
            Self::PermissionDenied(_) => ErrorCode::PermissionDenied,
            Self::Configuration(_) => ErrorCode::ConfigurationError,
            Self::Infrastructure(_) => ErrorCode::InfrastructureError,
            Self::Unexpected => ErrorCode::Unexpected,
        }
    }
}
