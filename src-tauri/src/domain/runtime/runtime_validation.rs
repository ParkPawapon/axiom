use super::php_runtime::{DetectedPhpBinary, PhpRuntime};

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeValidationResult {
    pub runtime: PhpRuntime,
    pub detected_binary: Option<DetectedPhpBinary>,
    pub valid: bool,
    pub status_message: String,
}
