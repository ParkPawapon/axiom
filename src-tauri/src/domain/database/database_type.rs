#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum DatabaseType {
    Mysql,
    Postgresql,
}
