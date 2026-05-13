import { Button } from "../../../shared/components/ui/button";
import { ServiceStatusRow } from "./service-status-row";
import type { ManagedService, ServiceAction } from "../types/service.types";

type BusyAction = ServiceAction | "check";

interface ServiceControlPanelProps {
  services: ManagedService[];
  isLoading: boolean;
  busyServiceId?: string;
  busyAction?: BusyAction;
  onRefresh: () => void;
  onCheck: (serviceId: string) => void;
  onStart: (serviceId: string) => void;
  onStop: (serviceId: string) => void;
  onRestart: (serviceId: string) => void;
}

export function ServiceControlPanel({
  busyAction,
  busyServiceId,
  isLoading,
  onCheck,
  onRefresh,
  onRestart,
  onStart,
  onStop,
  services,
}: ServiceControlPanelProps) {
  return (
    <section className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3 border border-voicebox-border bg-voicebox-surface p-4">
        <div>
          <h2 className="font-display text-2xl uppercase leading-none">Service Control</h2>
          <p className="mt-2 max-w-3xl text-sm text-voicebox-secondary">
            Backend-backed lifecycle control for MySQL, PostgreSQL, reverse proxy, and Docker.
            Actions stay disabled until a supported launchd label, Windows service, or Docker
            boundary is detected.
          </p>
        </div>
        <Button disabled={isLoading} onClick={onRefresh} variant="secondary">
          Refresh
        </Button>
      </div>

      <div className="grid gap-3">
        {services.map((service) => (
          <ServiceStatusRow
            busyAction={busyServiceId === service.id ? busyAction : undefined}
            key={service.id}
            onCheck={onCheck}
            onRestart={onRestart}
            onStart={onStart}
            onStop={onStop}
            service={service}
          />
        ))}
      </div>
    </section>
  );
}
