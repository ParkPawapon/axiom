use chrono::Utc;
use uuid::Uuid;

use crate::domain::security::audit_log::{AuditLogEntry, AuditSeverity};
use crate::ports::audit_logger::AuditLogger;
use crate::shared::result::app_result::AppResult;

pub fn record_security_audit(
    audit_logger: &dyn AuditLogger,
    operation: &str,
    resource: &str,
    severity: AuditSeverity,
    status: &str,
    message: &str,
) -> AppResult<()> {
    audit_logger.record(AuditLogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        actor: "local-desktop-user".to_string(),
        operation: operation.to_string(),
        resource: resource.to_string(),
        severity,
        status: status.to_string(),
        message: message.to_string(),
    })
}
