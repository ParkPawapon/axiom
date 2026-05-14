import { Chip } from "../../../shared/components/ui/chip";
import type { SecurityPermissionStatus } from "../types/security.types";

interface SecurityStatusGridProps {
  status?: SecurityPermissionStatus;
}

export function SecurityStatusGrid({ status }: SecurityStatusGridProps) {
  if (!status) {
    return null;
  }

  const items = [
    ["Hosts writable", status.hostFileWritable],
    ["Certificate store", status.certificateStoreAvailable],
    ["Audit log writable", status.auditLogWritable],
    ["Elevation supported", status.elevationSupported],
  ] as const;

  return (
    <section className="grid gap-3 border-2 border-voicebox-black bg-white p-4">
      <div>
        <p className="font-mono text-xs uppercase text-voicebox-secondary">Permission status</p>
        <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
          Security Boundary
        </h2>
      </div>
      <div className="grid gap-2 md:grid-cols-2">
        {items.map(([label, enabled]) => (
          <div
            className="flex items-center justify-between gap-3 border border-voicebox-border bg-voicebox-surface p-3"
            key={label}
          >
            <span className="text-sm font-bold text-voicebox-black">{label}</span>
            <Chip tone={enabled ? "success" : "warning"}>{enabled ? "Ready" : "Restricted"}</Chip>
          </div>
        ))}
      </div>
      <dl className="grid gap-2 font-mono text-xs text-voicebox-secondary">
        <div>
          <dt className="uppercase text-voicebox-tertiary">Hosts file</dt>
          <dd className="break-all">{status.hostsFilePath}</dd>
        </div>
        <div>
          <dt className="uppercase text-voicebox-tertiary">Local CA</dt>
          <dd className="break-all">{status.certificateAuthorityPath}</dd>
        </div>
      </dl>
    </section>
  );
}
