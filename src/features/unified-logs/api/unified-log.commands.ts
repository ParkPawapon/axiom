import { readProjectLogs } from "../../logs/api/log.commands";
import type { LogEntry } from "../../logs/types/log.types";
import { listProjects } from "../../projects/api/project.commands";
import type { Project } from "../../projects/types/project.types";
import { readProjectDockerLogs } from "../../services/api/docker.commands";
import type {
  UnifiedLogReadResult,
  UnifiedLogSource,
  UnifiedLogSourceId,
} from "../types/unified-log.types";

export const unifiedLogSources: UnifiedLogSource[] = [
  {
    id: "php",
    label: "PHP",
    available: true,
    statusMessage: "Reads sanitized PHP process logs through the Rust backend.",
  },
  {
    id: "docker",
    label: "Docker",
    available: true,
    statusMessage: "Reads project Docker logs through the backend Docker boundary.",
  },
  {
    id: "app",
    label: "App",
    available: false,
    statusMessage: "Application log reader is not exposed to the UI yet.",
  },
  {
    id: "mysql",
    label: "MySQL",
    available: false,
    statusMessage: "MySQL log reader is not connected to the unified view yet.",
  },
  {
    id: "postgresql",
    label: "PostgreSQL",
    available: false,
    statusMessage: "PostgreSQL log reader is not connected to the unified view yet.",
  },
  {
    id: "proxy",
    label: "Proxy",
    available: false,
    statusMessage: "Reverse proxy log reader is not connected to the unified view yet.",
  },
];

export function listUnifiedLogProjects() {
  return listProjects();
}

export async function readUnifiedLogs(
  sourceId: UnifiedLogSourceId,
  project: Project,
  tailCount: number,
  query?: string,
): Promise<UnifiedLogReadResult> {
  if (sourceId === "php") {
    const result = await readProjectLogs(project.id, tailCount, query);

    return {
      sourceId,
      entries: result.entries,
      truncated: result.truncated,
      statusMessage: result.statusMessage,
    };
  }

  if (sourceId === "docker") {
    const result = await readProjectDockerLogs(project.id, tailCount);
    const normalizedQuery = query?.trim().toLowerCase();
    const entries = result.lines
      .map<LogEntry>((line, index) => ({
        id: `docker-${index}`,
        lineNumber: index + 1,
        level: inferLogLevel(line),
        source: "docker",
        message: line,
        raw: line,
      }))
      .filter((entry) =>
        normalizedQuery ? entry.message.toLowerCase().includes(normalizedQuery) : true,
      );

    return {
      sourceId,
      entries,
      truncated: result.truncated,
      statusMessage: result.statusMessage,
    };
  }

  return {
    sourceId,
    entries: [],
    truncated: false,
    statusMessage:
      unifiedLogSources.find((source) => source.id === sourceId)?.statusMessage ??
      "This log source is not connected yet.",
  };
}

function inferLogLevel(line: string): LogEntry["level"] {
  const normalized = line.toLowerCase();

  if (normalized.includes("error") || normalized.includes("failed")) {
    return "error";
  }

  if (normalized.includes("warn")) {
    return "warn";
  }

  if (normalized.includes("debug")) {
    return "debug";
  }

  return "info";
}
