import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";

export function DatabasesPage() {
  return (
    <PageShell
      title="Databases"
      description="Future MySQL and PostgreSQL configuration surfaces will stay separate from service execution."
    >
      <EmptyState
        title="Database management not implemented"
        description="No database drivers, connections, service checks, or credentials are active in this scaffold."
      />
    </PageShell>
  );
}
