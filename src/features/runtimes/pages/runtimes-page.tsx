import { useCallback, useEffect, useMemo, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import { getProjectPhpVersion, listProjects } from "../../projects/api/project.commands";
import type {
  PhpVersionOption,
  Project,
  ProjectPhpVersionConfig,
} from "../../projects/types/project.types";
import { PhpVersionList } from "../components/php-version-list";
import { RuntimePathCard } from "../components/runtime-path-card";

interface ProjectRuntimeView {
  project: Project;
  config: ProjectPhpVersionConfig;
}

export function RuntimesPage() {
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isLoading, setIsLoading] = useState(true);
  const [runtimeViews, setRuntimeViews] = useState<ProjectRuntimeView[]>([]);

  const loadRuntimes = useCallback(async () => {
    setIsLoading(true);
    setErrorMessage(undefined);

    try {
      const projects = await listProjects();
      const views = await Promise.all(
        projects.map(async (project) => ({
          project,
          config: await getProjectPhpVersion(project.id),
        })),
      );

      setRuntimeViews(views);
    } catch (error) {
      setErrorMessage(
        getErrorMessage(error, "Runtime inventory could not be loaded from the desktop backend."),
      );
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadRuntimes();
  }, [loadRuntimes]);

  const versionCatalog = useMemo(() => {
    const versionMap = new Map<string, PhpVersionOption>();

    for (const view of runtimeViews) {
      for (const version of view.config.availablePhpVersions) {
        const current = versionMap.get(version.version);

        versionMap.set(version.version, {
          ...version,
          installed: Boolean(current?.installed || version.installed),
          binaryDisplayName: current?.binaryDisplayName ?? version.binaryDisplayName,
        });
      }
    }

    return Array.from(versionMap.values()).sort((left, right) =>
      right.version.localeCompare(left.version, undefined, { numeric: true }),
    );
  }, [runtimeViews]);

  return (
    <PageShell
      title="Runtimes"
      description="Per-project PHP runtime visibility from the same backend boundary used by project start and switch actions."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {isLoading ? <LoadingState label="Loading PHP runtime inventory" /> : null}

      {!isLoading && runtimeViews.length === 0 ? (
        <EmptyState
          title="No project runtimes"
          description="Add a project first, then select an installed PHP binary for that project."
        />
      ) : null}

      {runtimeViews.length > 0 ? (
        <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_24rem]">
          <PhpVersionList versions={versionCatalog} />
          <section className="grid content-start gap-3">
            {runtimeViews.map((view) => (
              <RuntimePathCard
                binary={view.config.selectedPhpBinary}
                key={view.project.id}
                projectName={view.project.name}
                selectedVersion={view.config.selectedPhpVersion}
              />
            ))}
          </section>
        </div>
      ) : null}
    </PageShell>
  );
}
