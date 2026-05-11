use crate::shared::error::app_error::AppError;

pub type AppResult<T> = Result<T, AppError>;
