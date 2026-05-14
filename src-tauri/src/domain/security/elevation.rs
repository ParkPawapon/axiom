#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionElevationKind {
    CertificateTrust,
    HostFileWrite,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionElevationRequest {
    pub kind: PermissionElevationKind,
    pub title: String,
    pub reason: String,
    pub command_preview: Vec<String>,
    pub requires_admin: bool,
    pub status_message: String,
}
