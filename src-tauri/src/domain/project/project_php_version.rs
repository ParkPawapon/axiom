use crate::domain::runtime::php_runtime::{DetectedPhpBinary, PhpRuntime};
use crate::domain::runtime::runtime_path::RuntimePath;
use crate::domain::runtime::runtime_version::RuntimeVersion;

use super::project_id::ProjectId;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPhpVersionConfig {
    pub project_id: ProjectId,
    pub selected_php_version: RuntimeVersion,
    pub selected_php_binary: Option<DetectedPhpBinary>,
    pub available_php_versions: Vec<PhpRuntime>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPhpRuntimeSelection {
    pub php_version: RuntimeVersion,
    pub php_binary_path: RuntimePath,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPhpInstallPlan {
    pub project_id: ProjectId,
    pub php_version: RuntimeVersion,
    pub requires_manual_confirmation: bool,
    pub warning_message: String,
    pub status_message: String,
}
