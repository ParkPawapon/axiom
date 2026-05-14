export type PermissionElevationKind = "certificateTrust" | "hostFileWrite";
export type CertificateTrustStatus = "missing" | "pending" | "trusted";
export type AuditSeverity = "error" | "info" | "warning";

export interface PermissionElevationRequest {
  readonly kind: PermissionElevationKind;
  readonly title: string;
  readonly reason: string;
  readonly commandPreview: string[];
  readonly requiresAdmin: boolean;
  readonly statusMessage: string;
}

export interface SecurityPermissionStatus {
  readonly hostsFilePath: string;
  readonly hostFileWritable: boolean;
  readonly certificateStoreAvailable: boolean;
  readonly certificateAuthorityPath: string;
  readonly auditLogWritable: boolean;
  readonly elevationSupported: boolean;
  readonly statusMessage: string;
}

export interface HostFileEntry {
  readonly domain: string;
  readonly address: string;
}

export interface HostFileUpdateResult {
  readonly entry: HostFileEntry;
  readonly hostsFilePath: string;
  readonly backupPath?: string | null;
  readonly preparedHostsPath?: string | null;
  readonly updated: boolean;
  readonly requiresElevation: boolean;
  readonly elevation?: PermissionElevationRequest | null;
  readonly statusMessage: string;
}

export interface LocalCertificate {
  readonly domain: string;
  readonly certificatePath: string;
  readonly privateKeyPath: string;
  readonly certificateAuthorityPath: string;
  readonly opensslConfigPath: string;
  readonly issuedAt: string;
  readonly statusMessage: string;
}

export interface CertificateTrustResult {
  readonly certificateAuthorityPath: string;
  readonly status: CertificateTrustStatus;
  readonly requiresElevation: boolean;
  readonly elevation?: PermissionElevationRequest | null;
  readonly statusMessage: string;
}

export interface AuditLogEntry {
  readonly id: string;
  readonly timestamp: string;
  readonly actor: string;
  readonly operation: string;
  readonly resource: string;
  readonly severity: AuditSeverity;
  readonly status: string;
  readonly message: string;
}

export interface AuditLogReadResult {
  readonly entries: AuditLogEntry[];
  readonly returnedEntries: number;
  readonly retentionDays: number;
  readonly logFile: string;
  readonly truncated: boolean;
  readonly statusMessage: string;
}

export interface AuditLogRetentionResult {
  readonly removedEntries: number;
  readonly retainedEntries: number;
  readonly retentionDays: number;
  readonly logFile: string;
  readonly statusMessage: string;
}
