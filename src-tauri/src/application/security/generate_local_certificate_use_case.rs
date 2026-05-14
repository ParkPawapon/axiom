use crate::domain::networking::ssl_certificate::LocalCertificate;
use crate::domain::security::audit_log::AuditSeverity;
use crate::ports::audit_logger::AuditLogger;
use crate::ports::certificate_manager::CertificateManager;
use crate::shared::result::app_result::AppResult;

use super::audit_support::record_security_audit;

pub fn generate_local_certificate(
    certificate_manager: &dyn CertificateManager,
    audit_logger: &dyn AuditLogger,
    domain: &str,
) -> AppResult<LocalCertificate> {
    let certificate = certificate_manager.generate_local_certificate(domain)?;

    record_security_audit(
        audit_logger,
        "local_certificate_generate",
        &certificate.domain,
        AuditSeverity::Info,
        "completed",
        &certificate.status_message,
    )?;

    Ok(certificate)
}
