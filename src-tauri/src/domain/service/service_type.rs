#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ServiceType {
    Php,
    Mysql,
    Postgresql,
    ReverseProxy,
    Docker,
}
