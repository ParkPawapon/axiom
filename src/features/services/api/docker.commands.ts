import { invokeTauriCommand } from "../../../core/api/tauri-client";
import type {
  DockerComposeProfile,
  DockerDiagnosticsReport,
  DockerImagePinResolutionReport,
  DockerProjectActionResult,
  DockerProjectComposePlan,
  DockerProjectImageOverride,
  DockerProjectLogReadResult,
  DockerProjectResourceLimits,
  DockerProjectRuntimeStatus,
  DockerProjectVolumeLifecycleResult,
} from "../types/docker.types";

export function getDockerDiagnostics() {
  return invokeTauriCommand<DockerDiagnosticsReport>("get_docker_diagnostics");
}

export function generateProjectDockerCompose(
  projectId: string,
  profiles: DockerComposeProfile[],
  imageOverrides: DockerProjectImageOverride[],
  resourceLimits: DockerProjectResourceLimits,
) {
  return invokeTauriCommand<DockerProjectComposePlan>("generate_project_docker_compose", {
    imageOverrides,
    projectId,
    profiles,
    resourceLimits,
  });
}

export function resolveProjectDockerImagePins(
  projectId: string,
  profiles: DockerComposeProfile[],
  imageOverrides: DockerProjectImageOverride[],
  resourceLimits: DockerProjectResourceLimits,
) {
  return invokeTauriCommand<DockerImagePinResolutionReport>("resolve_project_docker_image_pins", {
    imageOverrides,
    projectId,
    profiles,
    resourceLimits,
  });
}

export function getProjectDockerStatus(projectId: string) {
  return invokeTauriCommand<DockerProjectRuntimeStatus>("get_project_docker_status", {
    projectId,
  });
}

export function startProjectDockerServices(
  projectId: string,
  profiles: DockerComposeProfile[],
  imageOverrides: DockerProjectImageOverride[],
  resourceLimits: DockerProjectResourceLimits,
) {
  return invokeTauriCommand<DockerProjectActionResult>("start_project_docker_services", {
    imageOverrides,
    projectId,
    profiles,
    resourceLimits,
  });
}

export function stopProjectDockerServices(projectId: string) {
  return invokeTauriCommand<DockerProjectActionResult>("stop_project_docker_services", {
    projectId,
  });
}

export function restartProjectDockerServices(
  projectId: string,
  profiles: DockerComposeProfile[],
  imageOverrides: DockerProjectImageOverride[],
  resourceLimits: DockerProjectResourceLimits,
) {
  return invokeTauriCommand<DockerProjectActionResult>("restart_project_docker_services", {
    imageOverrides,
    projectId,
    profiles,
    resourceLimits,
  });
}

export function ensureProjectDockerVolumes(
  projectId: string,
  profiles: DockerComposeProfile[],
  imageOverrides: DockerProjectImageOverride[],
  resourceLimits: DockerProjectResourceLimits,
) {
  return invokeTauriCommand<DockerProjectVolumeLifecycleResult>("ensure_project_docker_volumes", {
    imageOverrides,
    projectId,
    profiles,
    resourceLimits,
  });
}

export function removeProjectDockerVolumes(projectId: string) {
  return invokeTauriCommand<DockerProjectVolumeLifecycleResult>("remove_project_docker_volumes", {
    projectId,
  });
}

export function readProjectDockerLogs(projectId: string, tailLines: number) {
  return invokeTauriCommand<DockerProjectLogReadResult>("read_project_docker_logs", {
    projectId,
    tailLines,
  });
}
