import { SetupStepPanel } from "./setup-step-panel";
import type { SetupDiagnosticStep } from "../types/setup-wizard.types";

export function DatabaseStep({ step }: { step: SetupDiagnosticStep }) {
  return <SetupStepPanel helper="Provision only the databases this project actually needs." step={step} />;
}
