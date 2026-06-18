import { StatusBadge } from "../../../shared/components/ui/status-badge";
import type { SetupDiagnosticStep } from "../types/setup-wizard.types";
import { statusLabel, statusTone } from "../../control-center/utils/map-service-status";

interface SetupStepperProps {
  activeStepId: string;
  steps: SetupDiagnosticStep[];
  onSelectStep: (stepId: string) => void;
}

export function SetupStepper({ activeStepId, steps, onSelectStep }: SetupStepperProps) {
  return (
    <nav className="grid gap-2" aria-label="Setup steps">
      {steps.map((step, index) => (
        <button
          aria-current={step.id === activeStepId ? "step" : undefined}
          className={`grid gap-2 border p-3 text-left ${
            step.id === activeStepId
              ? "border-voicebox-black bg-voicebox-black text-white"
              : "border-voicebox-border bg-white text-voicebox-black"
          }`}
          key={step.id}
          onClick={() => onSelectStep(step.id)}
          type="button"
        >
          <span className="font-mono text-xs uppercase">Step {index + 1}</span>
          <span className="font-display text-xl uppercase leading-none">{step.title}</span>
          <StatusBadge label={statusLabel(step.status)} tone={statusTone(step.status)} />
        </button>
      ))}
    </nav>
  );
}
