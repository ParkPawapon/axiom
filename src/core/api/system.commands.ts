import { invokeTauriCommand } from "./tauri-client";
import type { DockerCheckResult, PermissionCheckResult, PortCheckResult } from "../types/system.types";

export function checkPort(port: number) {
  return invokeTauriCommand<PortCheckResult>("check_port", { port });
}

export function checkDocker() {
  return invokeTauriCommand<DockerCheckResult>("check_docker");
}

export function checkPermissions() {
  return invokeTauriCommand<PermissionCheckResult>("check_permissions");
}
