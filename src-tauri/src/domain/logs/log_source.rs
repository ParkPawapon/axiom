#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum LogSource {
    Application,
    Php,
    Mysql,
    Postgresql,
    ReverseProxy,
}
