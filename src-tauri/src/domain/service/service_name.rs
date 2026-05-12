#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ServiceName(pub String);

impl ServiceName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}
