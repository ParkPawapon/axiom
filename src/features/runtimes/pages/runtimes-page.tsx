import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";

export function RuntimesPage() {
  return (
    <PageShell title="Runtimes" description="Future PHP runtime discovery and validation boundary.">
      <EmptyState
        title="Runtime detection not implemented"
        description="No PHP versions are scanned or executed by this scaffold."
      />
    </PageShell>
  );
}
