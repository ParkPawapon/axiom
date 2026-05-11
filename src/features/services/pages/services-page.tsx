import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";

export function ServicesPage() {
  return (
    <PageShell
      title="Services"
      description="Future PHP, database, proxy, SSL, and Docker service boundaries will be implemented here."
    >
      <EmptyState
        title="Service control disabled"
        description="Start, stop, restart, health checks, and process execution are not part of this architecture scaffold."
      />
    </PageShell>
  );
}
