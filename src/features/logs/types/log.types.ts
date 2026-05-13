import type { ProjectPhpProcessState } from "../../projects/types/project.types";

export type LogLevel = "debug" | "error" | "info" | "warn";

export interface LogEntry {
  readonly id: string;
  readonly lineNumber: number;
  readonly level: LogLevel;
  readonly source: string;
  readonly message: string;
  readonly raw: string;
}

export interface ProjectLogReadResult {
  readonly projectId: string;
  readonly logFile: string;
  readonly entries: LogEntry[];
  readonly returnedLines: number;
  readonly fileSizeBytes: number;
  readonly truncated: boolean;
  readonly statusMessage: string;
}

export interface ProjectLogSource {
  readonly projectId: string;
  readonly projectName: string;
  readonly processState: ProjectPhpProcessState;
  readonly logFile?: string;
  readonly statusMessage: string;
}
