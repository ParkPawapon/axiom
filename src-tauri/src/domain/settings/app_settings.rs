use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub app_name: String,
    pub theme: AppTheme,
    pub php_binary_search_paths: Vec<String>,
    pub docker_context: Option<String>,
    pub default_php_port: u16,
    pub default_mysql_port: u16,
    pub default_postgres_port: u16,
    pub audit_log_enabled: bool,
    pub updated_at: DateTime<Utc>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            app_name: "AxiomPHP".to_string(),
            theme: AppTheme::VoiceboxLight,
            php_binary_search_paths: Vec::new(),
            docker_context: None,
            default_php_port: 8080,
            default_mysql_port: 3306,
            default_postgres_port: 5432,
            audit_log_enabled: true,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AppTheme {
    VoiceboxLight,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsUpdate {
    pub theme: Option<AppTheme>,
    pub php_binary_search_paths: Option<Vec<String>>,
    pub docker_context: Option<Option<String>>,
    pub default_php_port: Option<u16>,
    pub default_mysql_port: Option<u16>,
    pub default_postgres_port: Option<u16>,
    pub audit_log_enabled: Option<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsUpdateResult {
    pub settings: AppSettings,
    pub status_message: String,
}
