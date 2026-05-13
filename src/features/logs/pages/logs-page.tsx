import { useCallback, useEffect, useMemo, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import { getProjectPhpProcessStatus, listProjects } from "../../projects/api/project.commands";
import { readProjectLogs } from "../api/log.commands";
import { LogFilterBar } from "../components/log-filter-bar";
import { LogSourceTabs } from "../components/log-source-tabs";
import { LogViewer } from "../components/log-viewer";
import type { ProjectLogReadResult, ProjectLogSource } from "../types/log.types";

export function LogsPage() {
  const [activeSourceId, setActiveSourceId] = useState<string>();
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isLogsLoading, setIsLogsLoading] = useState(false);
  const [isSourcesLoading, setIsSourcesLoading] = useState(true);
  const [logResult, setLogResult] = useState<ProjectLogReadResult>();
  const [maxLines, setMaxLines] = useState(300);
  const [query, setQuery] = useState("");
  const [sources, setSources] = useState<ProjectLogSource[]>([]);

  const loadLogSources = useCallback(async () => {
    setIsSourcesLoading(true);
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
      setIsSourcesLoading(false);
    }
  }, []);

  const activeSource = useMemo(
    () => sources.find((source) => source.projectId === activeSourceId),
    [activeSourceId, sources],
  );

  const loadActiveLog = useCallback(async () => {
    if (!activeSource) {
      setLogResult(undefined);
      return;
    }

    setIsLogsLoading(true);
    setErrorMessage(undefined);

    try {
      setLogResult(await readProjectLogs(activeSource.projectId, maxLines, query));
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "Project log file could not be read safely."));
    } finally {
      setIsLogsLoading(false);
    }
  }, [activeSource, maxLines, query]);

  const refreshLogs = useCallback(async () => {
    await loadLogSources();
    await loadActiveLog();
  }, [loadActiveLog, loadLogSources]);

  useEffect(() => {
    void loadLogSources();
  }, [loadLogSources]);

  useEffect(() => {
    void loadActiveLog();
  }, [loadActiveLog]);

  const isLoading = isSourcesLoading || isLogsLoading;

  return (
    <PageShell
      title="Logs"
      description="Backend-read project PHP process logs. The frontend requests a bounded tail from the Rust log reader and never reads arbitrary filesystem paths."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {isSourcesLoading ? <LoadingState label="Loading project log sources" /> : null}

      {sources.length > 0 ? (
        <section className="grid gap-4">
          <LogSourceTabs
            activeSourceId={activeSourceId}
            sources={sources}
            onSelect={setActiveSourceId}
          />
          <LogFilterBar
            isLoading={isLoading}
            maxLines={maxLines}
            query={query}
            sourceState={activeSource?.processState ?? "unavailable"}
            onMaxLinesChange={setMaxLines}
            onQueryChange={setQuery}
            onRefresh={() => void refreshLogs()}
          />
          {isLogsLoading ? <LoadingState label="Reading project log file" /> : null}
          <LogViewer result={logResult} source={activeSource} />
        </section>
      ) : null}

      {!isSourcesLoading && sources.length === 0 ? (
        <EmptyState
          title="No project log sources"
          description="Add a project and start its PHP process to create a backend-managed log source."
        />
      ) : null}
    </PageShell>
  );
}
