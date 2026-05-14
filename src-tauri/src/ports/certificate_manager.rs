use crate::domain::networking::ssl_certificate::{CertificateTrustResult, LocalCertificate};
use crate::shared::result::app_result::AppResult;

pub trait CertificateManager: Send + Sync {
    fn generate_local_certificate(&self, domain: &str) -> AppResult<LocalCertificate>;

    fn inspect_trust_status(&self) -> AppResult<CertificateTrustResult>;

    fn trust_local_certificate_authority(&self) -> AppResult<CertificateTrustResult>;
}
