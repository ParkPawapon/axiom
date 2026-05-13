use uuid::Uuid;

use crate::domain::database::database_config::ProjectDatabaseProfile;
use crate::domain::database::database_type::DatabaseType;
use crate::domain::database::mysql_config::{
    DEFAULT_MYSQL_HOST, DEFAULT_MYSQL_PHPMYADMIN_BASE_URL,
};
use crate::domain::database::postgres_config::DEFAULT_POSTGRES_HOST;
use crate::domain::project::project_id::ProjectId;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub const DATABASE_SECRET_NAMESPACE: &str = "database";

pub fn default_host(database_type: DatabaseType) -> &'static str {
    match database_type {
        DatabaseType::Mysql => DEFAULT_MYSQL_HOST,
        DatabaseType::Postgresql => DEFAULT_POSTGRES_HOST,
    }
}

pub fn database_display_name(database_type: DatabaseType) -> &'static str {
    match database_type {
        DatabaseType::Mysql => "MySQL",
        DatabaseType::Postgresql => "PostgreSQL",
    }
}

pub fn database_name(project_id: &ProjectId, database_type: DatabaseType) -> String {
    truncate_identifier(
        &format!(
            "ax_{}_{}",
            identifier_fragment(&project_id.0),
            database_type.as_key()
        ),
        60,
    )
}

pub fn username(project_id: &ProjectId) -> String {
    truncate_identifier(&format!("ax_{}", identifier_fragment(&project_id.0)), 28)
}

pub fn secret_key(profile: &ProjectDatabaseProfile) -> String {
    format!(
        "{}:{}",
        profile.project_id.0,
        profile.database_type.as_key()
    )
}

pub fn generate_password() -> String {
    format!(
        "Ax{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4()
            .simple()
            .to_string()
            .chars()
            .take(12)
            .collect::<String>()
    )
}

pub fn admin_url(database_type: DatabaseType, database_name: &str) -> Option<String> {
    match database_type {
        DatabaseType::Mysql => {
            let base_url = env_value("AXIOM_PHPMYADMIN_BASE_URL")
                .unwrap_or_else(|| DEFAULT_MYSQL_PHPMYADMIN_BASE_URL.to_string());

            if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
                return None;
            }

            Some(format!(
                "{}?db={}",
                base_url.trim_end_matches('/'),
                database_name
            ))
        }
        DatabaseType::Postgresql => None,
    }
}

pub fn env_value(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn sanitize_migration_name(name: &str) -> AppResult<String> {
    let mut sanitized = String::new();
    let mut previous_was_underscore = false;

    for character in name.trim().chars() {
        if character.is_ascii_alphanumeric() {
            sanitized.push(character.to_ascii_lowercase());
            previous_was_underscore = false;
        } else if !previous_was_underscore && !sanitized.is_empty() {
            sanitized.push('_');
            previous_was_underscore = true;
        }

        if sanitized.len() >= 48 {
            break;
        }
    }

    let sanitized = sanitized.trim_matches('_').to_string();

    if sanitized.is_empty() {
        return Err(AppError::Validation(
            "migration name must contain at least one letter or number".to_string(),
        ));
    }

    Ok(sanitized)
}

pub fn escape_sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

pub fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn identifier_fragment(value: &str) -> String {
    let mut fragment = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();

    while fragment.contains("__") {
        fragment = fragment.replace("__", "_");
    }

    fragment.trim_matches('_').to_string()
}

fn truncate_identifier(value: &str, max_len: usize) -> String {
    let trimmed = value.trim_matches('_');

    if trimmed.len() <= max_len {
        return trimmed.to_string();
    }

    trimmed.chars().take(max_len).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_safe_database_identifiers() {
        let project_id = ProjectId("my-project-123".to_string());

        assert_eq!(
            database_name(&project_id, DatabaseType::Mysql),
            "ax_my_project_123_mysql"
        );
        assert_eq!(username(&project_id), "ax_my_project_123");
    }

    #[test]
    fn rejects_empty_migration_names() {
        let result = sanitize_migration_name("../");

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn sanitizes_migration_names() {
        let migration_name = sanitize_migration_name("Create Users Table").expect("valid name");

        assert_eq!(migration_name, "create_users_table");
    }
}
