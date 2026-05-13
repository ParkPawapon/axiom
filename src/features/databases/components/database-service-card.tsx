import { StatusBadge } from "../../../shared/components/ui/status-badge";
import { formatServiceStatus } from "../../../shared/utils/format-service-status";
import type { ManagedService } from "../../services/types/service.types";
import { getServiceStatusTone } from "../../services/utils/service-status-tone";

interface DatabaseServiceCardProps {
  service: ManagedService;
}

export function DatabaseServiceCard({ service }: DatabaseServiceCardProps) {
  return (
    <article className="border-2 border-voicebox-black bg-white p-5">
      <div className="flex items-start justify-between gap-3 border-b border-voicebox-border pb-4">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">
            {service.serviceType}
          </p>
          <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
            {service.name}
          </h2>
        </div>
        <StatusBadge
          label={formatServiceStatus(service.status)}
          tone={getServiceStatusTone(service.status)}
        />
      </div>
      <p className="mt-4 text-sm text-voicebox-secondary">{service.description}</p>
      <p className="mt-4 border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
        {service.statusMessage}
      </p>
    </article>
  );
}
