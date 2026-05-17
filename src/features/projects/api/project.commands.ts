import { invokeTauriCommand } from "../../../core/api/tauri-client";
import type {
  Project,
  ProjectDockerActionResult,
  ProjectDockerStatus,
  ProjectPhpInstallPlan,
  ProjectPhpInstallResult,
  ProjectPhpProcessActionResult,
  ProjectPhpProcessStatus,
  ProjectPhpVersionConfig,
} from "../types/project.types";

export const projectCommands = {} as const;

export function listProjects() {
  return invokeTauriCommand<Project[]>("list_projects");
}

export function getProject(projectId: string) {
  return invokeTauriCommand<Project>("get_project", { projectId });
}

export function createProject(name: string, documentRoot: string) {
  return invokeTauriCommand<Project>("create_project", { name, documentRoot });
}

export function updateProject(projectId: string, name: string, documentRoot: string) {
  return invokeTauriCommand<Project>("update_project", { projectId, name, documentRoot });
}

export function deleteProject(projectId: string) {
  return invokeTauriCommand<void>("delete_project", { projectId });
}

export function validateProjectPath(documentRoot: string) {
  return invokeTauriCommand<string>("validate_project_path", { documentRoot });
}

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

export function getProjectPhpProcessStatus(projectId: string) {
  return invokeTauriCommand<ProjectPhpProcessStatus>("get_project_php_process_status", {
    projectId,
  });
}

export function startProjectPhpProcess(projectId: string) {
  return invokeTauriCommand<ProjectPhpProcessStatus>("start_project_php_process", { projectId });
}

export function startProjectPhpProcesses(projectIds: string[]) {
  return invokeTauriCommand<ProjectPhpProcessActionResult[]>("start_project_php_processes", {
    projectIds,
  });
}

export function stopProjectPhpProcess(projectId: string) {
  return invokeTauriCommand<ProjectPhpProcessStatus>("stop_project_php_process", { projectId });
}

export function stopProjectPhpProcesses(projectIds: string[]) {
  return invokeTauriCommand<ProjectPhpProcessActionResult[]>("stop_project_php_processes", {
    projectIds,
  });
}

export function restartProjectPhpProcess(projectId: string) {
  return invokeTauriCommand<ProjectPhpProcessStatus>("restart_project_php_process", { projectId });
}

export function restartProjectPhpProcesses(projectIds: string[]) {
  return invokeTauriCommand<ProjectPhpProcessActionResult[]>("restart_project_php_processes", {
    projectIds,
  });
}

export function generateProjectDockerCompose(projectId: string) {
  return invokeTauriCommand<ProjectDockerActionResult>("generate_project_docker_compose", {
    projectId,
  });
}

export function getProjectDockerStatus(projectId: string) {
  return invokeTauriCommand<ProjectDockerStatus>("get_project_docker_status", { projectId });
}

export function startProjectDocker(projectId: string) {
  return invokeTauriCommand<ProjectDockerActionResult>("start_project_docker", { projectId });
}

export function stopProjectDocker(projectId: string) {
  return invokeTauriCommand<ProjectDockerActionResult>("stop_project_docker", { projectId });
}

export function restartProjectDocker(projectId: string) {
  return invokeTauriCommand<ProjectDockerActionResult>("restart_project_docker", { projectId });
}
