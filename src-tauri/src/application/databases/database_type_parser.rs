use crate::domain::database::database_type::DatabaseType;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn parse_database_type(database_type: &str) -> AppResult<DatabaseType> {
    match database_type.trim().to_ascii_lowercase().as_str() {
        "mysql" => Ok(DatabaseType::Mysql),
        "postgres" | "postgresql" => Ok(DatabaseType::Postgresql),
        _ => Err(AppError::Validation(
            "database type must be mysql or postgresql".to_string(),
        )),
    }
}
