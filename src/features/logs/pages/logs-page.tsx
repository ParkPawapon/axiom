import { useCallback, useEffect, useMemo, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import { getProjectPhpProcessStatus, listProjects } from "../../projects/api/project.commands";
import { LogFilterBar } from "../components/log-filter-bar";
import { LogSourceTabs } from "../components/log-source-tabs";
import { LogViewer } from "../components/log-viewer";
import type { ProjectLogSource } from "../types/log.types";

export function LogsPage() {
  const [activeSourceId, setActiveSourceId] = useState<string>();
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isLoading, setIsLoading] = useState(true);
  const [query, setQuery] = useState("");
  const [sources, setSources] = useState<ProjectLogSource[]>([]);

  const loadLogSources = useCallback(async () => {
    setIsLoading(true);
    setErrorMessage(undefined);

    try {
      const projects = await listProjects();
      const nextSources = await Promise.all(
        projects.map(async (project) => {
          const status = await getProjectPhpProcessStatus(project.id);

          return {
            projectId: project.id,
            projectName: project.name,
            processState: status.state,
            logFile: status.logFile,
            statusMessage: status.statusMessage,
          };
        }),
      );

      setSources(nextSources);
      setActiveSourceId((currentSourceId) =>
        currentSourceId && nextSources.some((source) => source.projectId === currentSourceId)
          ? currentSourceId
          : nextSources[0]?.projectId,
      );
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "Log sources could not be loaded safely."));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadLogSources();
  }, [loadLogSources]);

  const activeSource = useMemo(
    () => sources.find((source) => source.projectId === activeSourceId),
    [activeSourceId, sources],
  );

  return (
    <PageShell
      title="Logs"
      description="Project process log metadata from the backend registry. File streaming remains gated until the log reader boundary is implemented."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {isLoading ? <LoadingState label="Loading project log sources" /> : null}

      {sources.length > 0 ? (
        <section className="grid gap-4">
          <LogSourceTabs
            activeSourceId={activeSourceId}
            sources={sources}
            onSelect={setActiveSourceId}
          />
          <LogFilterBar
            query={query}
            sourceState={activeSource?.processState ?? "unavailable"}
            onQueryChange={setQuery}
          />
          <LogViewer query={query} source={activeSource} />
        </section>
      ) : null}

      {!isLoading && sources.length === 0 ? (
        <EmptyState
          title="No project log sources"
          description="Add a project and start its PHP process to create a backend-managed log source."
        />
      ) : null}
    </PageShell>
  );
}
