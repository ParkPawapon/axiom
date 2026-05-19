use chrono::{DateTime, Utc};

use crate::domain::project::project_id::ProjectId;

#[derive(
    Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize,
)]
#[serde(rename_all = "camelCase")]
pub enum DockerComposeProfile {
    Mailpit,
    Mysql,
    Php,
    Postgresql,
    Redis,
    ReverseProxy,
}

impl DockerComposeProfile {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::Mailpit => "mailpit",
            Self::Mysql => "mysql",
            Self::Php => "php",
            Self::Postgresql => "postgresql",
            Self::Redis => "redis",
            Self::ReverseProxy => "reverseProxy",
        }
    }

    pub fn compose_profile(self) -> &'static str {
        match self {
            Self::Mailpit => "mailpit",
            Self::Mysql => "mysql",
            Self::Php => "php",
            Self::Postgresql => "postgresql",
            Self::Redis => "redis",
            Self::ReverseProxy => "reverse-proxy",
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProjectComposeRequest {
    pub project_id: ProjectId,
    pub profiles: Vec<DockerComposeProfile>,
    #[serde(default)]
    pub image_overrides: Vec<DockerProjectImageOverride>,
    #[serde(default)]
    pub resource_limits: DockerProjectResourceLimits,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProjectImageOverride {
    pub profile: DockerComposeProfile,
    pub image: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProjectResourceLimits {
    pub cpus: Option<f32>,
    pub memory_mb: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerRegistryTrustMetadata {
    pub registry: String,
    pub repository: String,
    pub reference: String,
    pub digest: String,
    pub media_type: String,
    pub platform_count: usize,
    pub allowed_registry: bool,
    pub status_message: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerImagePinResolution {
    pub profile: DockerComposeProfile,
    pub source_image: String,
    pub pinned_image: String,
    pub metadata: DockerRegistryTrustMetadata,
    pub status_message: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerImagePinResolutionReport {
    pub resolutions: Vec<DockerImagePinResolution>,
    pub diagnostics: Vec<String>,
    pub status_message: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerImageTrustEvaluation {
    pub profile: DockerComposeProfile,
    pub image: String,
    pub pinned_by_digest: bool,
    pub registry_allowed: bool,
    pub metadata_verified: bool,
    pub allowed: bool,
    pub metadata: Option<DockerRegistryTrustMetadata>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProjectServicePlan {
    pub profile: DockerComposeProfile,
    pub service_name: String,
    pub image: String,
    pub host_port: Option<u16>,
    pub container_port: Option<u16>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProjectVolumePlan {
    pub name: String,
    pub service_name: String,
    pub mount_path: String,
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProjectComposePlan {
    pub project_id: ProjectId,
    pub project_name: String,
    pub compose_project_name: String,
    pub compose_file_path: String,
    pub compose_file_written: bool,
    pub env_file_path: String,
    pub reverse_proxy_config_path: Option<String>,
    pub profiles: Vec<DockerComposeProfile>,
    pub services: Vec<DockerProjectServicePlan>,
    pub volumes: Vec<DockerProjectVolumePlan>,
    pub image_trust: Vec<DockerImageTrustEvaluation>,
    pub resource_limits: DockerProjectResourceLimits,
    pub diagnostics: Vec<String>,
    pub generated_at: DateTime<Utc>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProjectContainerStatus {
    pub name: String,
    pub service_name: String,
    pub state: String,
    pub status: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProjectRuntimeStatus {
    pub project_id: ProjectId,
    pub compose_project_name: String,
    pub engine_running: bool,
    pub compose_file_exists: bool,
    pub containers: Vec<DockerProjectContainerStatus>,
    pub volumes: Vec<DockerProjectVolumePlan>,
    pub diagnostics: Vec<String>,
    pub checked_at: DateTime<Utc>,
    pub status_message: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProjectActionResult {
    pub project_id: ProjectId,
    pub action: String,
    pub plan: DockerProjectComposePlan,
    pub runtime: DockerProjectRuntimeStatus,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProjectLogReadResult {
    pub project_id: ProjectId,
    pub lines: Vec<String>,
    pub truncated: bool,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerProjectVolumeLifecycleResult {
    pub project_id: ProjectId,
    pub volumes: Vec<DockerProjectVolumePlan>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerDiagnosticCheck {
    pub name: String,
    pub healthy: bool,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerDiagnosticsReport {
    pub cli_found: bool,
    pub engine_running: bool,
    pub compose_available: bool,
    pub docker_context: Option<String>,
    pub checks: Vec<DockerDiagnosticCheck>,
    pub status_message: String,
}
