use tauri::State;

use crate::application::security::{
    generate_local_certificate_use_case, get_security_status_use_case, prune_audit_log_use_case,
    read_audit_log_use_case, trust_certificate_use_case, update_hosts_file_use_case,
};
use crate::bootstrap::app_state::AppState;
use crate::domain::networking::host_entry::HostFileUpdateResult;
use crate::domain::networking::ssl_certificate::{CertificateTrustResult, LocalCertificate};
use crate::domain::security::audit_log::{AuditLogReadResult, AuditLogRetentionResult};
use crate::domain::security::security_status::SecurityPermissionStatus;
use crate::shared::error::command_error_mapper::{map_command_error, CommandErrorPayload};

#[tauri::command]
pub fn get_security_status(
    state: State<'_, AppState>,
) -> Result<SecurityPermissionStatus, CommandErrorPayload> {
    get_security_status_use_case::get_security_status(state.permission_manager()).map_err(|error| {
        tracing::warn!(?error, "security status command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn update_hosts_file(
    state: State<'_, AppState>,
    domain: String,
    address: String,
) -> Result<HostFileUpdateResult, CommandErrorPayload> {
    update_hosts_file_use_case::update_hosts_file(
        state.hosts_file_manager(),
        state.audit_logger(),
        &domain,
        &address,
    )
    .map_err(|error| {
        tracing::warn!(?error, "hosts file update command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn generate_local_certificate(
    state: State<'_, AppState>,
    domain: String,
) -> Result<LocalCertificate, CommandErrorPayload> {
    generate_local_certificate_use_case::generate_local_certificate(
        state.certificate_manager(),
        state.audit_logger(),
        &domain,
    )
    .map_err(|error| {
        tracing::warn!(?error, "local certificate generation command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn inspect_certificate_trust(
    state: State<'_, AppState>,
) -> Result<CertificateTrustResult, CommandErrorPayload> {
    trust_certificate_use_case::inspect_certificate_trust(state.certificate_manager()).map_err(
        |error| {
            tracing::warn!(?error, "certificate trust inspection command failed");
            map_command_error(&error)
        },
    )
}

#[tauri::command]
pub fn trust_certificate_authority(
    state: State<'_, AppState>,
) -> Result<CertificateTrustResult, CommandErrorPayload> {
    trust_certificate_use_case::trust_certificate_authority(
        state.certificate_manager(),
        state.audit_logger(),
    )
    .map_err(|error| {
        tracing::warn!(?error, "certificate trust command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn read_audit_log(
    state: State<'_, AppState>,
    max_entries: Option<usize>,
) -> Result<AuditLogReadResult, CommandErrorPayload> {
    read_audit_log_use_case::read_audit_log(state.audit_logger(), max_entries).map_err(|error| {
        tracing::warn!(?error, "audit log read command failed");
        map_command_error(&error)
    })
}

#[tauri::command]
pub fn prune_audit_log(
    state: State<'_, AppState>,
    retention_days: u16,
) -> Result<AuditLogRetentionResult, CommandErrorPayload> {
    prune_audit_log_use_case::prune_audit_log(state.audit_logger(), retention_days).map_err(
        |error| {
            tracing::warn!(?error, "audit log retention command failed");
            map_command_error(&error)
        },
    )
}
