import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";

export function LogsPage() {
  return (
    <PageShell
      title="Logs"
      description="Future log readers and stream adapters will be attached here."
    >
      <EmptyState
        title="Logs unavailable"
        description="No application, PHP, MySQL, PostgreSQL, proxy, or Docker logs are read by this scaffold."
      />
    </PageShell>
  );
}
