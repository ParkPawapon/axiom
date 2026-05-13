#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceType {
    Docker,
    Mysql,
    Php,
    Postgresql,
    ReverseProxy,
}
