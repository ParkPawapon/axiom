use super::project_id::ProjectId;
use super::project_path::ProjectPath;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub path: ProjectPath,
}
