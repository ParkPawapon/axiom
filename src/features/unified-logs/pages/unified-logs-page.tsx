import { useCallback, useEffect, useMemo, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { Select } from "../../../shared/components/ui/select";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import type { LogEntry } from "../../logs/types/log.types";
import type { Project } from "../../projects/types/project.types";
import {
  listUnifiedLogProjects,
  readUnifiedLogs,
  unifiedLogSources,
} from "../api/unified-log.commands";
import { LogSearchBar } from "../components/log-search-bar";
import { LogSourceFilter } from "../components/log-source-filter";
import { UnifiedLogViewer } from "../components/unified-log-viewer";
import type {
  UnifiedLogSeverityFilter,
  UnifiedLogSourceId,
} from "../types/unified-log.types";

export function UnifiedLogsPage() {
  const [activeSourceId, setActiveSourceId] = useState<UnifiedLogSourceId>("php");
  const [entries, setEntries] = useState<LogEntry[]>([]);
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isLoading, setIsLoading] = useState(true);
  const [projects, setProjects] = useState<Project[]>([]);
  const [query, setQuery] = useState("");
  const [selectedProjectId, setSelectedProjectId] = useState<string>();
  const [severity, setSeverity] = useState<UnifiedLogSeverityFilter>("all");
  const [statusMessage, setStatusMessage] = useState("Select a project to read logs.");
  const [tailCount, setTailCount] = useState(100);
  const [truncated, setTruncated] = useState(false);

  const selectedProject = useMemo(
    () => projects.find((project) => project.id === selectedProjectId) ?? projects[0],
    [projects, selectedProjectId],
  );
  const activeSource = unifiedLogSources.find((source) => source.id === activeSourceId);
  const filteredEntries = useMemo(
    () =>
      entries.filter((entry) => {
        if (severity !== "all" && entry.level !== severity) {
          return false;
        }

        return true;
      }),
    [entries, severity],
  );

  useEffect(() => {
    async function loadProjects() {
      try {
        const nextProjects = await listUnifiedLogProjects();
        setProjects(nextProjects);
        setSelectedProjectId((currentProjectId) =>
          currentProjectId && nextProjects.some((project) => project.id === currentProjectId)
            ? currentProjectId
            : nextProjects[0]?.id,
        );
      } catch (error) {
        setErrorMessage(getErrorMessage(error, "Projects could not be loaded for logs."));
      }
    }

    void loadProjects();
  }, []);

  const loadLogs = useCallback(async () => {
    setIsLoading(true);
    setErrorMessage(undefined);

    try {
      if (!selectedProject) {
        setEntries([]);
        setStatusMessage("Add a project before reading logs.");
        setTruncated(false);
        return;
      }

      const result = await readUnifiedLogs(activeSourceId, selectedProject, tailCount, query);
      setEntries(result.entries);
      setStatusMessage(result.statusMessage);
      setTruncated(result.truncated);
    } catch (error) {
      setEntries([]);
      setStatusMessage("Log read failed safely.");
      setTruncated(false);
      setErrorMessage(getErrorMessage(error, "Log read failed safely."));
    } finally {
      setIsLoading(false);
    }
  }, [activeSourceId, query, selectedProject, tailCount]);

  useEffect(() => {
    void loadLogs();
  }, [loadLogs]);

  return (
    <PageShell
      title="Logs"
      description="Unified sanitized log access for projects and supported backend readers."
    >
      <div className="grid gap-4">
        {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
        <section className="grid gap-4 border-2 border-voicebox-black bg-white p-4">
          <div className="grid gap-3 lg:grid-cols-[18rem_minmax(0,1fr)]">
            <label className="grid gap-1 text-sm font-semibold text-voicebox-secondary">
              Project
              <Select
                aria-label="Select project for logs"
                disabled={projects.length === 0}
                onChange={(event) => setSelectedProjectId(event.target.value)}
                value={selectedProject?.id ?? ""}
              >
                {projects.map((project) => (
                  <option key={project.id} value={project.id}>
                    {project.name}
                  </option>
                ))}
              </Select>
            </label>
            <div className="grid gap-2">
              <LogSourceFilter
                activeSourceId={activeSourceId}
                onSourceChange={setActiveSourceId}
                sources={unifiedLogSources}
              />
              <p className="text-sm text-voicebox-secondary">
                {activeSource?.statusMessage ?? "Select a source."}
              </p>
            </div>
          </div>
          <LogSearchBar
            onQueryChange={setQuery}
            onSeverityChange={setSeverity}
            onTailCountChange={setTailCount}
            query={query}
            severity={severity}
            tailCount={tailCount}
          />
        </section>

        {isLoading ? <LoadingState label="Reading sanitized logs" /> : null}
        <UnifiedLogViewer
          entries={filteredEntries}
          statusMessage={statusMessage}
          truncated={truncated}
        />
      </div>
    </PageShell>
  );
}
