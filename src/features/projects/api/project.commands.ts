import { invokeTauriCommand } from "../../../core/api/tauri-client";
import type {
  ProjectPhpInstallPlan,
  ProjectPhpInstallResult,
  ProjectPhpVersionConfig,
} from "../types/project.types";

export const projectCommands = {} as const;

export function getProjectPhpVersion(projectId: string) {
  return invokeTauriCommand<ProjectPhpVersionConfig>("get_project_php_version", { projectId });
}

export function selectProjectPhpVersion(projectId: string, phpVersion: string) {
  return invokeTauriCommand<ProjectPhpVersionConfig>("select_project_php_version", {
    projectId,
    phpVersion,
  });
}

export function requestProjectPhpInstall(projectId: string, phpVersion: string) {
  return invokeTauriCommand<ProjectPhpInstallPlan>("request_project_php_install", {
    projectId,
    phpVersion,
  });
}

export function installProjectPhpRuntime(projectId: string, phpVersion: string) {
  return invokeTauriCommand<ProjectPhpInstallResult>("install_project_php_runtime", {
    projectId,
    phpVersion,
  });
}
