import { invokeTauriCommand } from "../../../core/api/tauri-client";
import type { ControlCenterSummary, SetupDiagnosticsReport } from "../types/control-center.types";

export function getControlCenterSummary(selectedProjectId?: string) {
  return invokeTauriCommand<ControlCenterSummary>("get_control_center_summary", {
    selectedProjectId,
  });
}

export function getSetupDiagnostics(selectedProjectId?: string) {
  return invokeTauriCommand<SetupDiagnosticsReport>("get_setup_diagnostics", {
    selectedProjectId,
  });
}
