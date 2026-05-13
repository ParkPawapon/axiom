import type { ServiceStatus } from "../../features/services/types/service.types";

const labels: Record<ServiceStatus, string> = {
  detected: "Detected",
  failed: "Failed",
  notConfigured: "Not configured",
  running: "Running",
  stopped: "Stopped",
};

export function formatServiceStatus(status: ServiceStatus) {
  return labels[status];
}
