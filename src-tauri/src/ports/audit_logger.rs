use crate::domain::security::audit_log::{
    AuditLogEntry, AuditLogReadResult, AuditLogRetentionResult,
};
use crate::shared::result::app_result::AppResult;

pub trait AuditLogger: Send + Sync {
    fn record(&self, entry: AuditLogEntry) -> AppResult<()>;

    fn read(&self, max_entries: usize) -> AppResult<AuditLogReadResult>;

    fn prune(&self, retention_days: u16) -> AppResult<AuditLogRetentionResult>;
}
