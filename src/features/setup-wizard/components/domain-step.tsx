import { SetupStepPanel } from "./setup-step-panel";
import type { SetupDiagnosticStep } from "../types/setup-wizard.types";

export function DomainStep({ step }: { step: SetupDiagnosticStep }) {
  return <SetupStepPanel helper="Local domain and HTTPS setup remains explicit and permission-aware." step={step} />;
}
