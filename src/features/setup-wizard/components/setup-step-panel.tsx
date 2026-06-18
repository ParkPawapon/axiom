import { StatusBadge } from "../../../shared/components/ui/status-badge";
import { diagnosticSeverityTone } from "../../control-center/utils/map-diagnostic-severity";
import { statusLabel, statusTone } from "../../control-center/utils/map-service-status";
import type { SetupDiagnosticStep } from "../types/setup-wizard.types";

interface SetupStepPanelProps {
  helper: string;
  step: SetupDiagnosticStep;
}

export function SetupStepPanel({ helper, step }: SetupStepPanelProps) {
  return (
    <section className="border-2 border-voicebox-black bg-white p-5">
      <div className="flex flex-wrap items-start justify-between gap-3 border-b border-voicebox-border pb-4">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">Guided Setup</p>
          <h2 className="mt-1 font-display text-3xl uppercase leading-none text-voicebox-black">
            {step.title}
          </h2>
          <p className="mt-3 text-sm text-voicebox-secondary">{helper}</p>
        </div>
        <StatusBadge label={statusLabel(step.status)} tone={statusTone(step.status)} />
      </div>

      <div className="mt-4 grid gap-3">
        {step.diagnostics.length === 0 ? (
          <p className="border border-voicebox-border bg-voicebox-surface p-4 text-sm text-voicebox-secondary">
            This step is ready. No action is required.
          </p>
        ) : (
          step.diagnostics.map((diagnostic) => (
            <article className="border border-voicebox-border bg-voicebox-surface p-4" key={diagnostic.id}>
              <div className="flex flex-wrap items-start justify-between gap-3">
                <h3 className="font-semibold text-voicebox-primary">{diagnostic.title}</h3>
                <StatusBadge
                  label={diagnostic.severity}
                  tone={diagnosticSeverityTone(diagnostic.severity)}
                />
              </div>
              <p className="mt-3 text-sm leading-relaxed text-voicebox-secondary">
                {diagnostic.message}
              </p>
              <p className="mt-3 border-l-2 border-voicebox-black pl-3 text-sm font-semibold">
                {diagnostic.nextStep}
              </p>
            </article>
          ))
        )}
      </div>
    </section>
  );
}
