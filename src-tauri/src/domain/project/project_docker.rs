use super::project_id::ProjectId;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDockerComposeConfig {
    pub project_id: ProjectId,
    pub compose_project_name: String,
    pub compose_file_path: String,
    pub project_runtime_dir: String,
    pub document_root: String,
    pub php_image: String,
    pub service_name: String,
    pub container_document_root: String,
    pub container_port: u16,
    pub status_message: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectDockerState {
    Failed,
    NotGenerated,
    Running,
    Stopped,
    Unavailable,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectDockerAction {
    Generate,
    Restart,
    Start,
    Stop,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDockerStatus {
    pub project_id: ProjectId,
    pub state: ProjectDockerState,
    pub compose_project_name: String,
    pub compose_file_path: Option<String>,
    pub service_name: String,
    pub container_id: Option<String>,
    pub published_port: Option<u16>,
    pub url: Option<String>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDockerActionResult {
    pub project_id: ProjectId,
    pub action: ProjectDockerAction,
    pub status: ProjectDockerStatus,
    pub message: String,
}
