import { StatusBadge } from "../../../shared/components/ui/status-badge";
import type { ControlCenterPortSummary } from "../types/control-center.types";
import { statusLabel, statusTone } from "../utils/map-service-status";

interface PortOverviewPanelProps {
  ports: ControlCenterPortSummary[];
}

export function PortOverviewPanel({ ports }: PortOverviewPanelProps) {
  return (
    <section className="border border-voicebox-border bg-white p-4">
      <h2 className="font-display text-2xl uppercase leading-none text-voicebox-black">Ports</h2>
      <div className="mt-4 grid gap-2">
        {ports.map((port) => (
          <div
            className="grid grid-cols-[5rem_1fr_auto] items-center gap-3 border border-voicebox-border bg-voicebox-surface p-3 text-sm"
            key={port.id}
          >
            <span className="font-semibold">{port.label}</span>
            <span className="font-mono text-xs text-voicebox-secondary">:{port.port}</span>
            <StatusBadge label={statusLabel(port.status)} tone={statusTone(port.status)} />
          </div>
        ))}
      </div>
    </section>
  );
}
