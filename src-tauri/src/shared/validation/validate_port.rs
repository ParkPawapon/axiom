use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn validate_port(port: u16) -> AppResult<u16> {
    if port == 0 {
        return Err(AppError::Validation("port must be greater than zero".to_string()));
    }

    Ok(port)
}
