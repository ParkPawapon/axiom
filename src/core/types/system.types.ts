import type { DockerDiagnosticsReport } from "../../features/services/types/docker.types";
import type { SecurityPermissionStatus } from "../../features/security/types/security.types";

export type PortCheckState = "available" | "inUse" | "unavailable";

export interface PortCheckResult {
  readonly port: number;
  readonly bindAddress: string;
  readonly state: PortCheckState;
  readonly checkedAt: string;
  readonly statusMessage: string;
}

export interface DockerCheckResult {
  readonly diagnostics: DockerDiagnosticsReport;
  readonly checkedAt: string;
  readonly statusMessage: string;
}

export interface PermissionCheckResult {
  readonly permissions: SecurityPermissionStatus;
  readonly checkedAt: string;
  readonly statusMessage: string;
}
