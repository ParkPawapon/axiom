import { invokeTauriCommand } from "../../../core/api/tauri-client";
import type { ManagedService, ServiceActionOutcome } from "../types/service.types";

export function listServices() {
  return invokeTauriCommand<ManagedService[]>("list_services");
}

export function getServiceStatus(serviceId: string) {
  return invokeTauriCommand<ManagedService>("get_service_status", { serviceId });
}

export function startService(serviceId: string) {
  return invokeTauriCommand<ServiceActionOutcome>("start_service", { serviceId });
}

export function stopService(serviceId: string) {
  return invokeTauriCommand<ServiceActionOutcome>("stop_service", { serviceId });
}

export function restartService(serviceId: string) {
  return invokeTauriCommand<ServiceActionOutcome>("restart_service", { serviceId });
}
