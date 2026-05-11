import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";

export function ProjectsPage() {
  return (
    <PageShell
      title="Projects"
      description="Future project-based PHP environment configuration will live here."
    >
      <EmptyState
        title="No project management yet"
        description="Project creation, document root selection, local domains, and environment profiles are intentionally not implemented in this scaffold."
      />
    </PageShell>
  );
}
