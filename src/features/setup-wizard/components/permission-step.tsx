import { SetupStepPanel } from "./setup-step-panel";
import type { SetupDiagnosticStep } from "../types/setup-wizard.types";

export function PermissionStep({ step }: { step: SetupDiagnosticStep }) {
  return <SetupStepPanel helper="Privileged changes must use prepared backend requests." step={step} />;
}
