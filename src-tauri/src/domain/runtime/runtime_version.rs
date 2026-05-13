use std::fmt;

use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeVersion(String);

impl RuntimeVersion {
    pub fn new(value: &str) -> AppResult<Self> {
        let value = value.trim();

        if !is_valid_runtime_version(value) {
            return Err(AppError::Validation(
                "runtime version must use a major.minor numeric format".to_string(),
            ));
        }

        Ok(Self(value.to_string()))
    }

    pub fn trusted(value: &str) -> Self {
        Self(value.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RuntimeVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

fn is_valid_runtime_version(value: &str) -> bool {
    let Some((major, minor)) = value.split_once('.') else {
        return false;
    };

    !major.is_empty()
        && !minor.is_empty()
        && major.bytes().all(|byte| byte.is_ascii_digit())
        && minor.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_major_minor_versions() {
        let version = RuntimeVersion::new("8.4").expect("version should validate");

        assert_eq!(version.as_str(), "8.4");
    }

    #[test]
    fn rejects_path_like_versions() {
        let result = RuntimeVersion::new("../8.4");

        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
