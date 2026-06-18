import type { StatusTone } from "../../../shared/components/ui/status-badge";
import type { ControlCenterStatus } from "../types/control-center.types";

export function statusLabel(status: ControlCenterStatus) {
  const labels: Record<ControlCenterStatus, string> = {
    blockedSafely: "Blocked safely",
    checking: "Checking",
    error: "Error",
    missing: "Missing",
    needsSetup: "Needs setup",
    ready: "Ready",
    running: "Running",
    stopped: "Stopped",
  };

  return labels[status];
}

export function statusTone(status: ControlCenterStatus): StatusTone {
  if (status === "ready" || status === "running") {
    return "success";
  }

  if (status === "needsSetup" || status === "stopped" || status === "checking") {
    return "warning";
  }

  if (status === "error" || status === "missing" || status === "blockedSafely") {
    return "error";
  }

  return "neutral";
}
