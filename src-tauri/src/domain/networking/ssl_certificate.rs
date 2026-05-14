use chrono::{DateTime, Utc};

use crate::domain::security::elevation::PermissionElevationRequest;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalCertificate {
    pub domain: String,
    pub certificate_path: String,
    pub private_key_path: String,
    pub certificate_authority_path: String,
    pub openssl_config_path: String,
    pub issued_at: DateTime<Utc>,
    pub status_message: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CertificateTrustStatus {
    Missing,
    Pending,
    Trusted,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateTrustResult {
    pub certificate_authority_path: String,
    pub status: CertificateTrustStatus,
    pub requires_elevation: bool,
    pub elevation: Option<PermissionElevationRequest>,
    pub status_message: String,
}
