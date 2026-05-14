use crate::domain::security::audit_log::AuditLogReadResult;
use crate::ports::audit_logger::AuditLogger;
use crate::shared::result::app_result::AppResult;

const DEFAULT_MAX_AUDIT_ENTRIES: usize = 200;
const MAX_AUDIT_ENTRIES: usize = 1_000;

pub fn read_audit_log(
    audit_logger: &dyn AuditLogger,
    max_entries: Option<usize>,
) -> AppResult<AuditLogReadResult> {
    audit_logger.read(
        max_entries
            .unwrap_or(DEFAULT_MAX_AUDIT_ENTRIES)
            .clamp(1, MAX_AUDIT_ENTRIES),
    )
}
