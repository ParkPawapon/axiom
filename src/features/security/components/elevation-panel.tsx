import type { PermissionElevationRequest } from "../types/security.types";

interface ElevationPanelProps {
  request: PermissionElevationRequest;
}

export function ElevationPanel({ request }: ElevationPanelProps) {
  return (
    <section className="grid gap-3 border-2 border-voicebox-warning bg-white p-4">
      <div>
        <p className="font-mono text-xs uppercase text-voicebox-warning">
          {request.requiresAdmin ? "Admin approval" : "User approval"}
        </p>
        <h3 className="mt-1 font-display text-xl uppercase leading-none text-voicebox-black">
          {request.title}
        </h3>
      </div>
      <p className="text-sm leading-relaxed text-voicebox-secondary">{request.reason}</p>
      <div className="grid gap-2">
        {request.commandPreview.map((command) => (
          <code
            className="block break-all border border-voicebox-border bg-voicebox-surface p-2 font-mono text-xs text-voicebox-black"
            key={command}
          >
            {command}
          </code>
        ))}
      </div>
      <p className="font-mono text-xs leading-relaxed text-voicebox-secondary">
        {request.statusMessage}
      </p>
    </section>
  );
}
