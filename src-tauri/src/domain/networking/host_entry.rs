use crate::domain::security::elevation::PermissionElevationRequest;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostFileEntry {
    pub domain: String,
    pub address: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostFileUpdateResult {
    pub entry: HostFileEntry,
    pub hosts_file_path: String,
    pub backup_path: Option<String>,
    pub prepared_hosts_path: Option<String>,
    pub updated: bool,
    pub requires_elevation: bool,
    pub elevation: Option<PermissionElevationRequest>,
    pub status_message: String,
}
