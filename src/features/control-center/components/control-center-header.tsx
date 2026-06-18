import { StatusBadge } from "../../../shared/components/ui/status-badge";
import type { ControlCenterSummary } from "../types/control-center.types";
import { statusLabel, statusTone } from "../utils/map-service-status";

interface ControlCenterHeaderProps {
  summary: ControlCenterSummary;
  onRefresh: () => void;
  isLoading: boolean;
}

export function ControlCenterHeader({ summary, onRefresh, isLoading }: ControlCenterHeaderProps) {
  return (
    <header className="grid gap-4 border-2 border-voicebox-black bg-white p-5 lg:grid-cols-[minmax(0,1fr)_auto]">
      <div>
        <p className="font-mono text-xs uppercase text-voicebox-secondary">AxiomPHP</p>
        <h1 className="mt-1 font-display text-4xl uppercase leading-none text-voicebox-black md:text-5xl">
          Control Center
        </h1>
        <p className="mt-3 max-w-3xl text-sm leading-relaxed text-voicebox-secondary">
          XAMPP-style daily controls with safe backend boundaries. Advanced service, Docker, and
          security details stay behind diagnostics.
        </p>
      </div>
      <div className="flex flex-col items-start gap-3 lg:items-end">
        <StatusBadge label={statusLabel(summary.status)} tone={statusTone(summary.status)} />
        <button
          className="border-2 border-voicebox-black px-4 py-2 text-sm font-bold disabled:cursor-not-allowed disabled:border-voicebox-border disabled:text-voicebox-tertiary"
          disabled={isLoading}
          onClick={onRefresh}
          type="button"
        >
          Refresh
        </button>
      </div>
    </header>
  );
}
