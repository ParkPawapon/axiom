import { StatusBadge } from "../../../shared/components/ui/status-badge";
import type { ControlCenterServiceSummary } from "../types/control-center.types";
import { statusLabel, statusTone } from "../utils/map-service-status";

interface ServiceStatusTileProps {
  service: ControlCenterServiceSummary;
}

export function ServiceStatusTile({ service }: ServiceStatusTileProps) {
  return (
    <article className="grid min-h-40 gap-3 border border-voicebox-border bg-white p-4">
      <div className="flex items-start justify-between gap-3">
        <h3 className="font-display text-xl uppercase leading-none text-voicebox-black">
          {service.label}
        </h3>
        <StatusBadge label={statusLabel(service.status)} tone={statusTone(service.status)} />
      </div>
      <p className="text-sm leading-relaxed text-voicebox-primary">{service.primaryDetail}</p>
      {service.secondaryDetail ? (
        <p className="break-words font-mono text-xs leading-relaxed text-voicebox-secondary">
          {service.secondaryDetail}
        </p>
      ) : null}
    </article>
  );
}
