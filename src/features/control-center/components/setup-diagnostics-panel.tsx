import { StatusBadge } from "../../../shared/components/ui/status-badge";
import type { SetupDiagnostic } from "../types/control-center.types";
import { diagnosticSeverityTone } from "../utils/map-diagnostic-severity";
import { statusLabel } from "../utils/map-service-status";

interface SetupDiagnosticsPanelProps {
  diagnostics: SetupDiagnostic[];
}

export function SetupDiagnosticsPanel({ diagnostics }: SetupDiagnosticsPanelProps) {
  return (
    <section className="border-2 border-voicebox-black bg-white p-4">
      <div className="mb-3 flex flex-wrap items-center justify-between gap-3">
        <h2 className="font-display text-2xl uppercase leading-none text-voicebox-black">
          Setup Diagnostics
        </h2>
        <StatusBadge
          label={diagnostics.length === 0 ? "Ready" : `${diagnostics.length} item(s)`}
          tone={diagnostics.length === 0 ? "success" : "warning"}
        />
      </div>
      <div className="grid gap-2">
        {diagnostics.length === 0 ? (
          <p className="border border-voicebox-border bg-voicebox-surface p-3 text-sm text-voicebox-secondary">
            No blocking setup items were reported.
          </p>
        ) : (
          diagnostics.map((diagnostic) => (
            <details className="border border-voicebox-border bg-voicebox-surface p-3" key={diagnostic.id}>
              <summary className="cursor-pointer list-none">
                <span className="flex flex-wrap items-center justify-between gap-3">
                  <span className="font-semibold text-voicebox-primary">{diagnostic.title}</span>
                  <StatusBadge
                    label={statusLabel(diagnostic.status)}
                    tone={diagnosticSeverityTone(diagnostic.severity)}
                  />
                </span>
              </summary>
              <p className="mt-3 text-sm leading-relaxed text-voicebox-secondary">
                {diagnostic.message}
              </p>
              <p className="mt-2 border-l-2 border-voicebox-black pl-3 text-sm font-semibold text-voicebox-primary">
                {diagnostic.nextStep}
              </p>
              {diagnostic.details ? (
                <p className="mt-2 break-words font-mono text-xs text-voicebox-secondary">
                  {diagnostic.details}
                </p>
              ) : null}
            </details>
          ))
        )}
      </div>
    </section>
  );
}
