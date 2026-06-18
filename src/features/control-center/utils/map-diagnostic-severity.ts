import type { StatusTone } from "../../../shared/components/ui/status-badge";
import type { ControlCenterSeverity } from "../types/control-center.types";

export function diagnosticSeverityTone(severity: ControlCenterSeverity): StatusTone {
  const tones: Record<ControlCenterSeverity, StatusTone> = {
    error: "error",
    info: "neutral",
    warning: "warning",
  };

  return tones[severity];
}
