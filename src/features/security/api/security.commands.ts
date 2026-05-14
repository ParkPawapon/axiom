import { invokeTauriCommand } from "../../../core/api/tauri-client";
import type {
  AuditLogReadResult,
  AuditLogRetentionResult,
  CertificateTrustResult,
  HostFileUpdateResult,
  LocalCertificate,
  SecurityPermissionStatus,
} from "../types/security.types";

export function getSecurityStatus() {
  return invokeTauriCommand<SecurityPermissionStatus>("get_security_status");
}

export function updateHostsFile(domain: string, address: string) {
  return invokeTauriCommand<HostFileUpdateResult>("update_hosts_file", {
    address,
    domain,
  });
}

export function generateLocalCertificate(domain: string) {
  return invokeTauriCommand<LocalCertificate>("generate_local_certificate", {
    domain,
  });
}

export function inspectCertificateTrust() {
  return invokeTauriCommand<CertificateTrustResult>("inspect_certificate_trust");
}

export function trustCertificateAuthority() {
  return invokeTauriCommand<CertificateTrustResult>("trust_certificate_authority");
}

export function readAuditLog(maxEntries: number) {
  return invokeTauriCommand<AuditLogReadResult>("read_audit_log", {
    maxEntries,
  });
}

export function pruneAuditLog(retentionDays: number) {
  return invokeTauriCommand<AuditLogRetentionResult>("prune_audit_log", {
    retentionDays,
  });
}
