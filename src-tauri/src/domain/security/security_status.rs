#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPermissionStatus {
    pub hosts_file_path: String,
    pub host_file_writable: bool,
    pub certificate_store_available: bool,
    pub certificate_authority_path: String,
    pub audit_log_writable: bool,
    pub elevation_supported: bool,
    pub status_message: String,
}
