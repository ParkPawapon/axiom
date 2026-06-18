import { SetupStepPanel } from "./setup-step-panel";
import type { SetupDiagnosticStep } from "../types/setup-wizard.types";

export function SystemCheckStep({ step }: { step: SetupDiagnosticStep }) {
  return <SetupStepPanel helper="Checks Docker, OS readiness, and core app status." step={step} />;
}
