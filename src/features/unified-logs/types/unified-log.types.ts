import type { LogEntry, LogLevel } from "../../logs/types/log.types";

export type UnifiedLogSourceId = "app" | "php" | "mysql" | "postgresql" | "proxy" | "docker";

export interface UnifiedLogSource {
  readonly id: UnifiedLogSourceId;
  readonly label: string;
  readonly available: boolean;
  readonly statusMessage: string;
}

export interface UnifiedLogReadResult {
  readonly sourceId: UnifiedLogSourceId;
  readonly entries: LogEntry[];
  readonly truncated: boolean;
  readonly statusMessage: string;
}

export type UnifiedLogSeverityFilter = "all" | LogLevel;
