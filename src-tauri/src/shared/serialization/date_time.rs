use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serializer};

use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub fn parse_rfc3339(value: &str) -> AppResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|error| AppError::Validation(format!("timestamp must be RFC3339: {error}")))
}

pub fn serialize<S>(value: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_rfc3339())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    DateTime::parse_from_rfc3339(&value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rfc3339_as_utc() {
        let parsed = parse_rfc3339("2026-05-20T10:00:00+07:00").expect("timestamp should parse");

        assert_eq!(parsed.to_rfc3339(), "2026-05-20T03:00:00+00:00");
    }
}
