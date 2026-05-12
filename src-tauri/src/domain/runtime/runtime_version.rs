use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct RuntimeVersion(pub String);

impl RuntimeVersion {
    pub fn new(version: &str) -> AppResult<Self> {
        let trimmed = version.trim();

        if trimmed.is_empty() {
            return Err(AppError::Validation(
                "runtime version must not be empty".to_string(),
            ));
        }

        if trimmed != version {
            return Err(AppError::Validation(
                "runtime version must not include leading or trailing whitespace".to_string(),
            ));
        }

        if trimmed.len() > 16 {
            return Err(AppError::Validation(
                "runtime version must be 16 characters or fewer".to_string(),
            ));
        }

        let is_safe = trimmed.bytes().all(|byte| {
            byte.is_ascii_digit() || byte == b'.' || byte.is_ascii_lowercase() || byte == b'-'
        });

        if !is_safe {
            return Err(AppError::Validation(
                "runtime version contains unsupported characters".to_string(),
            ));
        }

        Ok(Self(trimmed.to_string()))
    }

    pub fn trusted(version: &'static str) -> Self {
        Self(version.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_runtime_versions_with_whitespace() {
        let result = RuntimeVersion::new(" 8.5");

        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
