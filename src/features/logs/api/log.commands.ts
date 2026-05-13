import { invokeTauriCommand } from "../../../core/api/tauri-client";
import type { ProjectLogReadResult } from "../types/log.types";

export function readProjectLogs(projectId: string, maxLines: number, query?: string) {
  return invokeTauriCommand<ProjectLogReadResult>("read_project_logs", {
    maxLines,
    projectId,
    query,
  });
}
