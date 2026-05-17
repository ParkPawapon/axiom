export interface Project {
  readonly id: string;
  readonly name: string;
  readonly documentRoot: string;
  readonly createdAt: string;
  readonly updatedAt: string;
}

export interface ProjectDraft {
  readonly name: string;
  readonly documentRoot: string;
}

export type PhpVersionSupportPhase = "active" | "security" | "endOfLife";

export interface DetectedPhpBinary {
  readonly version: string;
  readonly path: string;
  readonly displayName: string;
}

export interface PhpVersionOption {
  readonly version: string;
  readonly label: string;
  readonly supportPhase: PhpVersionSupportPhase;
  readonly recommended: boolean;
  readonly installed: boolean;
  readonly binaryDisplayName?: string;
  readonly canSwitch: boolean;
  readonly requiresManualInstallConfirmation: boolean;
  readonly lifecycleWarning?: string;
}

export interface ProjectPhpVersionConfig {
  readonly projectId: string;
  readonly selectedPhpVersion: string;
  readonly selectedPhpBinary?: DetectedPhpBinary;
  readonly availablePhpVersions: PhpVersionOption[];
  readonly statusMessage: string;
}

export interface ProjectPhpInstallPlan {
  readonly projectId: string;
  readonly phpVersion: string;
  readonly requiresManualConfirmation: boolean;
  readonly provider?: PhpRuntimeInstallProvider;
  readonly packageName?: string;
  readonly warningMessage: string;
  readonly statusMessage: string;
}

export type PhpRuntimeInstallProvider = "homebrew" | "scoop";

export type PhpRuntimeInstallDiagnosticLevel = "info" | "warning";

export interface PhpRuntimeInstallDiagnostic {
  readonly level: PhpRuntimeInstallDiagnosticLevel;
  readonly code: string;
  readonly message: string;
}

export interface PhpRuntimeInstallRollback {
  readonly attempted: boolean;
  readonly succeeded: boolean;
  readonly message: string;
}

export interface ProjectPhpInstallResult {
  readonly projectId: string;
  readonly phpVersion: string;
  readonly provider: PhpRuntimeInstallProvider;
  readonly packageName: string;
  readonly selectedPhpBinary?: DetectedPhpBinary;
  readonly diagnostics: PhpRuntimeInstallDiagnostic[];
  readonly rollback?: PhpRuntimeInstallRollback;
  readonly statusMessage: string;
}

export type ProjectPhpProcessState = "failed" | "running" | "stopped";

export interface ProjectPhpProcessStatus {
  readonly projectId: string;
  readonly state: ProjectPhpProcessState;
  readonly pid?: number;
  readonly phpVersion?: string;
  readonly port?: number;
  readonly url?: string;
  readonly documentRoot?: string;
  readonly logFile?: string;
  readonly startedAt?: string;
  readonly statusMessage: string;
}

export interface ProjectPhpProcessActionResult {
  readonly projectId: string;
  readonly succeeded: boolean;
  readonly status?: ProjectPhpProcessStatus;
  readonly errorCode?: string;
  readonly errorMessage?: string;
}

export type ProjectDockerState = "failed" | "notGenerated" | "running" | "stopped" | "unavailable";
export type ProjectDockerAction = "generate" | "restart" | "start" | "stop";

export interface ProjectDockerStatus {
  readonly projectId: string;
  readonly state: ProjectDockerState;
  readonly composeProjectName: string;
  readonly composeFilePath?: string | null;
  readonly serviceName: string;
  readonly containerId?: string | null;
  readonly publishedPort?: number | null;
  readonly url?: string | null;
  readonly statusMessage: string;
}

export interface ProjectDockerActionResult {
  readonly projectId: string;
  readonly action: ProjectDockerAction;
  readonly status: ProjectDockerStatus;
  readonly message: string;
}
