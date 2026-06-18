import { SetupStepPanel } from "./setup-step-panel";
import type { SetupDiagnosticStep } from "../types/setup-wizard.types";

export function PhpVersionStep({ step }: { step: SetupDiagnosticStep }) {
  return <SetupStepPanel helper="Select a project PHP binary before using Start." step={step} />;
}
