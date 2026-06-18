import type { ControlCenterServiceSummary } from "../types/control-center.types";
import { ServiceStatusTile } from "./service-status-tile";

interface ServiceStatusGridProps {
  services: ControlCenterServiceSummary[];
}

export function ServiceStatusGrid({ services }: ServiceStatusGridProps) {
  return (
    <section className="grid gap-3 md:grid-cols-2 xl:grid-cols-3" aria-label="Service status">
      {services.map((service) => (
        <ServiceStatusTile key={service.id} service={service} />
      ))}
    </section>
  );
}
