import { useCallback, useEffect, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import {
  restartProjectPhpProcess,
  startProjectPhpProcess,
  stopProjectPhpProcess,
} from "../../projects/api/project.commands";
import { getControlCenterSummary } from "../api/control-center.commands";
import { ControlCenterHeader } from "../components/control-center-header";
import { PortOverviewPanel } from "../components/port-overview-panel";
import { ProjectQuickActions } from "../components/project-quick-actions";
import { ProjectSwitcher } from "../components/project-switcher";
import { SafeDefaultsPanel } from "../components/safe-defaults-panel";
import { ServiceStatusGrid } from "../components/service-status-grid";
import { SetupDiagnosticsPanel } from "../components/setup-diagnostics-panel";
import { UnifiedLogPreview } from "../components/unified-log-preview";
import type { ControlCenterSummary, QuickActionKind } from "../types/control-center.types";

export function ControlCenterPage() {
  const [summary, setSummary] = useState<ControlCenterSummary>();
  const [selectedProjectId, setSelectedProjectId] = useState<string>();
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isLoading, setIsLoading] = useState(true);
  const [isActionBusy, setIsActionBusy] = useState(false);

  const loadSummary = useCallback(async () => {
    setIsLoading(true);
    setErrorMessage(undefined);

    try {
      const nextSummary = await getControlCenterSummary(selectedProjectId);
      setSummary(nextSummary);
      setSelectedProjectId((currentProjectId) => currentProjectId ?? nextSummary.selectedProject?.id);
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "Control Center could not be loaded safely."));
    } finally {
      setIsLoading(false);
    }
  }, [selectedProjectId]);

  useEffect(() => {
    void loadSummary();
  }, [loadSummary]);

  const runProjectAction = useCallback(
    async (kind: QuickActionKind) => {
      if (!summary?.selectedProject) {
        return;
      }

      setIsActionBusy(true);
      setErrorMessage(undefined);

      try {
        if (kind === "startProject") {
          await startProjectPhpProcess(summary.selectedProject.id);
        } else if (kind === "stopProject") {
          await stopProjectPhpProcess(summary.selectedProject.id);
        } else if (kind === "restartProject") {
          await restartProjectPhpProcess(summary.selectedProject.id);
        }

        await loadSummary();
      } catch (error) {
        setErrorMessage(getErrorMessage(error, "Project action was blocked safely."));
      } finally {
        setIsActionBusy(false);
      }
    },
    [loadSummary, summary],
  );

  const navigateLogs = useCallback(() => {
    globalThis.location.hash = "logs";
  }, []);

  return (
    <PageShell
      title="Control Center"
      description="Daily PHP project controls with safe backend diagnostics."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {isLoading && !summary ? <LoadingState label="Loading Control Center" /> : null}

      {summary ? (
        <div className="grid gap-4">
          <ControlCenterHeader
            isLoading={isLoading}
            onRefresh={() => void loadSummary()}
            summary={summary}
          />
          <ProjectSwitcher
            onProjectChange={setSelectedProjectId}
            projects={summary.projects}
            selectedProject={summary.selectedProject}
          />
          <ProjectQuickActions
            actions={summary.quickActions}
            isBusy={isActionBusy}
            onNavigateLogs={navigateLogs}
            onProjectAction={runProjectAction}
          />
          <ServiceStatusGrid services={summary.services} />
          <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_22rem]">
            <SetupDiagnosticsPanel diagnostics={summary.diagnostics} />
            <div className="grid gap-4">
              <PortOverviewPanel ports={summary.ports} />
              <UnifiedLogPreview onOpenLogs={navigateLogs} preview={summary.logPreview} />
              <SafeDefaultsPanel />
            </div>
          </div>
        </div>
      ) : null}
    </PageShell>
  );
}
