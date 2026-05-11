import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";

export function SettingsPage() {
  return (
    <PageShell
      title="Settings"
      description="Future preferences, runtime paths, Docker settings, ports, and security configuration."
    >
      <EmptyState
        title="Settings are placeholders"
        description="No configuration is read, written, migrated, or persisted by this scaffold."
      />
    </PageShell>
  );
}
