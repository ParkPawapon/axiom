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
    pub provider: Option<PhpRuntimeInstallProvider>,
    pub package_name: Option<String>,
    pub warning_message: String,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPhpInstallResult {
    pub project_id: ProjectId,
    pub php_version: RuntimeVersion,
    pub provider: PhpRuntimeInstallProvider,
    pub package_name: String,
    pub selected_php_binary: Option<DetectedPhpBinary>,
    pub diagnostics: Vec<PhpRuntimeInstallDiagnostic>,
    pub rollback: Option<PhpRuntimeInstallRollback>,
    pub status_message: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PhpRuntimeInstallProvider {
    Homebrew,
    Scoop,
}

impl PhpRuntimeInstallProvider {
    pub fn label(self) -> &'static str {
        match self {
            Self::Homebrew => "Homebrew",
            Self::Scoop => "Scoop",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PhpRuntimeInstallDiagnosticLevel {
    Info,
    Warning,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpRuntimeInstallDiagnostic {
    pub level: PhpRuntimeInstallDiagnosticLevel,
    pub code: String,
    pub message: String,
}

impl PhpRuntimeInstallDiagnostic {
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: PhpRuntimeInstallDiagnosticLevel::Info,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: PhpRuntimeInstallDiagnosticLevel::Warning,
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpRuntimeInstallRollback {
    pub attempted: bool,
    pub succeeded: bool,
    pub message: String,
}
