use crate::domain::security::audit_log::{AuditLogRetentionResult, AuditSeverity};
use crate::ports::audit_logger::AuditLogger;
use crate::shared::error::app_error::AppError;
use crate::shared::result::app_result::AppResult;

use super::audit_support::record_security_audit;

pub fn prune_audit_log(
    audit_logger: &dyn AuditLogger,
    retention_days: u16,
) -> AppResult<AuditLogRetentionResult> {
    if !(1..=365).contains(&retention_days) {
        return Err(AppError::Validation(
            "audit log retention must be between 1 and 365 days".to_string(),
        ));
    }

    let result = audit_logger.prune(retention_days)?;

    record_security_audit(
        audit_logger,
        "audit_log_retention",
        &result.log_file,
        AuditSeverity::Info,
        "completed",
        &result.status_message,
    )?;

    Ok(result)
}
