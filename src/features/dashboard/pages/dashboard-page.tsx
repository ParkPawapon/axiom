import { PageShell } from "../../../shared/components/layout/page-shell";
import { Card } from "../../../shared/components/ui/card";
import { StatusBadge } from "../../../shared/components/ui/status-badge";

const placeholderPanels = ["PHP runtime", "MySQL", "PostgreSQL", "Port status"];

export function DashboardPage() {
  return (
    <PageShell
      title="Dashboard"
      description="A minimal desktop shell for the future local PHP development control center."
    >
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {placeholderPanels.map((panel) => (
          <Card aria-disabled="true" key={panel}>
            <div className="flex items-start justify-between gap-3">
              <h2 className="font-bold">{panel}</h2>
              <StatusBadge label="Not implemented" />
            </div>
            <p className="mt-6 text-sm text-voicebox-secondary">
              Placeholder panel. No service detection or process execution is active.
            </p>
          </Card>
        ))}
      </div>
      <section className="border-2 border-voicebox-black bg-white p-5">
        <p className="font-display text-2xl uppercase">Architecture scaffold only</p>
        <p className="mt-3 max-w-3xl text-sm text-voicebox-secondary">
          Backend command handlers, use cases, ports, infrastructure adapters, and platform adapters
          are prepared for future implementation.
        </p>
      </section>
    </PageShell>
  );
}
