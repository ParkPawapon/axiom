import { Button } from "../../../shared/components/ui/button";
import { StatusBadge, type StatusTone } from "../../../shared/components/ui/status-badge";
import { formatServiceStatus } from "../../../shared/utils/format-service-status";
import type { ManagedService, ServiceAction } from "../types/service.types";

type BusyAction = ServiceAction | "check";

interface ServiceStatusRowProps {
  service: ManagedService;
  busyAction?: BusyAction;
  onCheck: (serviceId: string) => void;
  onStart: (serviceId: string) => void;
  onStop: (serviceId: string) => void;
  onRestart: (serviceId: string) => void;
}

const statusTone: Record<ManagedService["status"], StatusTone> = {
  detected: "neutral",
  failed: "error",
  notConfigured: "warning",
  running: "success",
  stopped: "neutral",
};

export function ServiceStatusRow({
  service,
  busyAction,
  onCheck,
  onRestart,
  onStart,
  onStop,
}: ServiceStatusRowProps) {
  const isBusy = busyAction !== undefined;

  return (
    <article className="grid gap-4 border border-voicebox-border bg-white p-4 md:grid-cols-[minmax(0,1fr)_auto]">
      <div className="min-w-0">
        <div className="flex flex-wrap items-center gap-3">
          <h3 className="font-display text-xl uppercase leading-none text-voicebox-black">
            {service.name}
          </h3>
          <StatusBadge
            label={formatServiceStatus(service.status)}
            tone={statusTone[service.status]}
          />
        </div>
        <p className="mt-2 max-w-2xl text-sm text-voicebox-secondary">{service.description}</p>
        <p className="mt-3 border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
          {service.statusMessage}
        </p>
      </div>

      <div className="flex flex-wrap items-center gap-2 md:justify-end">
        <Button
          aria-label={`Check ${service.name} status`}
          disabled={isBusy}
          onClick={() => onCheck(service.id)}
          variant="secondary"
        >
          Check
        </Button>
        <Button
          aria-label={`Start ${service.name}`}
          disabled={!service.canStart || isBusy}
          onClick={() => onStart(service.id)}
        >
          Start
        </Button>
        <Button
          aria-label={`Stop ${service.name}`}
          disabled={!service.canStop || isBusy}
          onClick={() => onStop(service.id)}
          variant="secondary"
        >
          Stop
        </Button>
        <Button
          aria-label={`Restart ${service.name}`}
          disabled={!service.canRestart || isBusy}
          onClick={() => onRestart(service.id)}
          variant="secondary"
        >
          Restart
        </Button>
      </div>
    </article>
  );
}
