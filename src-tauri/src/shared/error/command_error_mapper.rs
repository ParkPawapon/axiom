use serde::Serialize;

use super::app_error::AppError;
use super::error_code::ErrorCode;

#[derive(Debug, Serialize)]
pub struct CommandErrorPayload {
    pub code: ErrorCode,
    pub message: String,
}

pub fn map_command_error(error: &AppError) -> CommandErrorPayload {
    CommandErrorPayload {
        code: error.code(),
        message: error.to_string(),
    }
}
