use crate::domain::networking::ssl_certificate::CertificateTrustResult;
use crate::domain::security::audit_log::AuditSeverity;
use crate::ports::audit_logger::AuditLogger;
use crate::ports::certificate_manager::CertificateManager;
use crate::shared::result::app_result::AppResult;

use super::audit_support::record_security_audit;

pub fn inspect_certificate_trust(
    certificate_manager: &dyn CertificateManager,
) -> AppResult<CertificateTrustResult> {
    certificate_manager.inspect_trust_status()
}

pub fn trust_certificate_authority(
    certificate_manager: &dyn CertificateManager,
    audit_logger: &dyn AuditLogger,
) -> AppResult<CertificateTrustResult> {
    let result = certificate_manager.trust_local_certificate_authority()?;

    record_security_audit(
        audit_logger,
        "certificate_trust",
        &result.certificate_authority_path,
        if result.requires_elevation {
            AuditSeverity::Warning
        } else {
            AuditSeverity::Info
        },
        if result.requires_elevation {
            "waiting_for_elevation"
        } else {
            "completed"
        },
        &result.status_message,
    )?;

    Ok(result)
}
