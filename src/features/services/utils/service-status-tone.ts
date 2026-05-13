import type { StatusTone } from "../../../shared/components/ui/status-badge";
import type { ServiceStatus } from "../types/service.types";

export function getServiceStatusTone(status: ServiceStatus): StatusTone {
  if (status === "running" || status === "detected") {
    return "success";
  }

  if (status === "failed") {
    return "error";
  }

  if (status === "notConfigured") {
    return "warning";
  }

  return "neutral";
}
