use crate::domain::runtime::php_runtime::PhpRuntime;
use crate::domain::runtime::runtime_version::RuntimeVersion;

use super::project_id::ProjectId;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPhpVersionConfig {
    pub project_id: ProjectId,
    pub selected_php_version: RuntimeVersion,
    pub available_php_versions: Vec<PhpRuntime>,
    pub status_message: String,
}
