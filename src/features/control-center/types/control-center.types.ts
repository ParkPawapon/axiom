import type { LogEntry } from "../../logs/types/log.types";

export type ControlCenterStatus =
  | "blockedSafely"
  | "checking"
  | "error"
  | "missing"
  | "needsSetup"
  | "ready"
  | "running"
  | "stopped";

export type ControlCenterSeverity = "error" | "info" | "warning";

export interface SetupDiagnostic {
  readonly id: string;
  readonly title: string;
  readonly status: ControlCenterStatus;
  readonly severity: ControlCenterSeverity;
  readonly message: string;
  readonly nextStep: string;
  readonly details?: string;
}

export interface SetupDiagnosticStep {
  readonly id: string;
  readonly title: string;
  readonly status: ControlCenterStatus;
  readonly diagnostics: SetupDiagnostic[];
}

export interface SetupDiagnosticsReport {
  readonly steps: SetupDiagnosticStep[];
  readonly ready: boolean;
  readonly statusMessage: string;
}

export interface ControlCenterProjectSummary {
  readonly id: string;
  readonly name: string;
  readonly documentRoot: string;
  readonly phpUrl?: string;
  readonly phpPort?: number;
  readonly phpVersion?: string;
}

export interface ControlCenterServiceSummary {
  readonly id: string;
  readonly label: string;
  readonly status: ControlCenterStatus;
  readonly primaryDetail: string;
  readonly secondaryDetail?: string;
  readonly canStart: boolean;
  readonly canStop: boolean;
  readonly canRestart: boolean;
}

export interface ControlCenterPortSummary {
  readonly id: string;
  readonly label: string;
  readonly port: number;
  readonly status: ControlCenterStatus;
  readonly statusMessage: string;
}

export interface ControlCenterLogPreview {
  readonly source: string;
  readonly entries: LogEntry[];
  readonly status: ControlCenterStatus;
  readonly statusMessage: string;
}

export type QuickActionKind =
  | "openBrowser"
  | "openFolder"
  | "openLogs"
  | "openPhpMyAdmin"
  | "restartProject"
  | "startProject"
  | "stopProject";

export interface QuickActionSummary {
  readonly id: string;
  readonly label: string;
  readonly kind: QuickActionKind;
  readonly status: ControlCenterStatus;
  readonly enabled: boolean;
  readonly disabledReason?: string;
  readonly target?: string;
}

export interface ControlCenterSummary {
  readonly projects: ControlCenterProjectSummary[];
  readonly selectedProject?: ControlCenterProjectSummary;
  readonly services: ControlCenterServiceSummary[];
  readonly diagnostics: SetupDiagnostic[];
  readonly quickActions: QuickActionSummary[];
  readonly ports: ControlCenterPortSummary[];
  readonly logPreview: ControlCenterLogPreview;
  readonly setup: SetupDiagnosticsReport;
  readonly generatedAt: string;
  readonly status: ControlCenterStatus;
  readonly statusMessage: string;
}
