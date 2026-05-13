import type { ProjectPhpProcessState } from "../../projects/types/project.types";

export interface ProjectLogSource {
  readonly projectId: string;
  readonly projectName: string;
  readonly processState: ProjectPhpProcessState;
  readonly logFile?: string;
  readonly statusMessage: string;
}
