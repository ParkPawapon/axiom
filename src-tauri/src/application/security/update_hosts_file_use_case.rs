use crate::domain::networking::host_entry::{HostFileEntry, HostFileUpdateResult};
use crate::domain::security::audit_log::AuditSeverity;
use crate::ports::audit_logger::AuditLogger;
use crate::ports::hosts_file_manager::HostsFileManager;
use crate::shared::result::app_result::AppResult;
use crate::shared::validation::validate_local_domain::{
    validate_local_domain, validate_loopback_address,
};

use super::audit_support::record_security_audit;

pub fn update_hosts_file(
    hosts_file_manager: &dyn HostsFileManager,
    audit_logger: &dyn AuditLogger,
    domain: &str,
    address: &str,
) -> AppResult<HostFileUpdateResult> {
    let entry = HostFileEntry {
        domain: validate_local_domain(domain)?,
        address: validate_loopback_address(address)?,
    };
    let result = hosts_file_manager.apply_entry(entry)?;
    let status = if result.updated {
        "completed"
    } else if result.requires_elevation {
        "waiting_for_elevation"
    } else {
        "unchanged"
    };

    record_security_audit(
        audit_logger,
        "hosts_file_update",
        &result.entry.domain,
        if result.requires_elevation {
            AuditSeverity::Warning
        } else {
            AuditSeverity::Info
        },
        status,
        &result.status_message,
    )?;

    Ok(result)
}
