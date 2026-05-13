#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DatabaseType {
    Mysql,
    Postgresql,
}

impl DatabaseType {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Mysql => "mysql",
            Self::Postgresql => "postgresql",
        }
    }

    pub fn default_port(self) -> u16 {
        match self {
            Self::Mysql => 3306,
            Self::Postgresql => 5432,
        }
    }
}
