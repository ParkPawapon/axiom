import { SetupStepPanel } from "./setup-step-panel";
import type { SetupDiagnosticStep } from "../types/setup-wizard.types";

export function SetupSummaryStep({ step }: { step: SetupDiagnosticStep }) {
  return <SetupStepPanel helper="Review remaining blockers before using the Control Center." step={step} />;
}
