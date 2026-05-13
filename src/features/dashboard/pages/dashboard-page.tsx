import { useCallback, useEffect, useMemo, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";
import { StatusBadge } from "../../../shared/components/ui/status-badge";
import { formatDate } from "../../../shared/utils/format-date";
import { formatServiceStatus } from "../../../shared/utils/format-service-status";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import {
  getProjectPhpProcessStatus,
  getProjectPhpVersion,
  listProjects,
} from "../../projects/api/project.commands";
import type {
  Project,
  ProjectPhpProcessStatus,
  ProjectPhpVersionConfig,
} from "../../projects/types/project.types";
import { listServices } from "../../services/api/service.commands";
import type { ManagedService } from "../../services/types/service.types";
import { getServiceStatusTone } from "../../services/utils/service-status-tone";
import { PortStatusCard } from "../components/port-status-card";
import { ProjectSummaryCard } from "../components/project-summary-card";
import { RuntimeHealthPanel } from "../components/runtime-health-panel";
import { ServiceOverviewCard } from "../components/service-overview-card";

interface DashboardRuntimeState {
  projectId: string;
  config?: ProjectPhpVersionConfig;
}

interface DashboardProcessState {
  projectId: string;
  status?: ProjectPhpProcessStatus;
}

const commandFailureMessage =
  "Dashboard data could not be loaded safely. Check the desktop backend status.";

export function DashboardPage() {
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isLoading, setIsLoading] = useState(true);
  const [processStates, setProcessStates] = useState<DashboardProcessState[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [runtimeStates, setRuntimeStates] = useState<DashboardRuntimeState[]>([]);
  const [services, setServices] = useState<ManagedService[]>([]);

  const loadDashboard = useCallback(async () => {
    setIsLoading(true);
    setErrorMessage(undefined);

    try {
      const [nextProjects, nextServices] = await Promise.all([listProjects(), listServices()]);
      const [nextRuntimeStates, nextProcessStates] = await Promise.all([
        Promise.all(
          nextProjects.map(async (project) => ({
            projectId: project.id,
            config: await getProjectPhpVersion(project.id),
          })),
        ),
        Promise.all(
          nextProjects.map(async (project) => ({
            projectId: project.id,
            status: await getProjectPhpProcessStatus(project.id),
          })),
        ),
      ]);

      setProjects(nextProjects);
      setServices(nextServices);
      setRuntimeStates(nextRuntimeStates);
      setProcessStates(nextProcessStates);
    } catch (error) {
      setErrorMessage(getErrorMessage(error, commandFailureMessage));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadDashboard();
  }, [loadDashboard]);

  const runningProcesses = useMemo(
    () =>
      processStates
        .map((processState) => processState.status)
        .filter(
          (status): status is ProjectPhpProcessStatus =>
            status !== undefined && status.state === "running",
        ),
    [processStates],
  );

  const configuredRuntimeCount = runtimeStates.filter(
    (runtimeState) => runtimeState.config?.selectedPhpBinary,
  ).length;
  const missingRuntimeCount = runtimeStates.filter(
    (runtimeState) => !runtimeState.config?.selectedPhpBinary,
  ).length;
  const runningServiceCount = services.filter((service) => service.status === "running").length;
  const failedServiceCount = services.filter((service) => service.status === "failed").length;

  return (
    <PageShell
      title="Dashboard"
      description="Real-time desktop overview from the Rust command boundary. Unsupported subsystems stay visible as guarded states instead of fake controls."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {isLoading ? <LoadingState label="Loading local environment overview" /> : null}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <ProjectSummaryCard
          configuredCount={configuredRuntimeCount}
          runningCount={runningProcesses.length}
          totalCount={projects.length}
        />
        <RuntimeHealthPanel
          missingBinaryCount={missingRuntimeCount}
          selectedVersionCount={configuredRuntimeCount}
          totalProjectCount={projects.length}
        />
        <ServiceOverviewCard
          failedCount={failedServiceCount}
          runningCount={runningServiceCount}
          totalCount={services.length}
        />
        <PortStatusCard
          ports={runningProcesses.flatMap((status) => (status.port ? [status.port] : []))}
        />
      </div>

      <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_24rem]">
        <section className="border-2 border-voicebox-black bg-white p-5">
          <div className="border-b border-voicebox-border pb-4">
            <p className="font-mono text-xs uppercase text-voicebox-secondary">Running Projects</p>
            <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
              PHP Processes
            </h2>
          </div>

          {runningProcesses.length === 0 ? (
            <div className="mt-5">
              <EmptyState
                title="No PHP process running"
                description="Start a configured project from Projects after selecting an installed PHP binary."
              />
            </div>
          ) : (
            <div className="mt-5 grid gap-3">
              {runningProcesses.map((status) => {
                const project = projects.find(
                  (currentProject) => currentProject.id === status.projectId,
                );

                return (
                  <article
                    className="grid gap-3 border border-voicebox-border bg-voicebox-surface p-4 md:grid-cols-[minmax(0,1fr)_auto]"
                    key={status.projectId}
                  >
                    <div className="min-w-0">
                      <h3 className="font-display text-xl uppercase leading-none text-voicebox-black">
                        {project?.name ?? status.projectId}
                      </h3>
                      <p className="mt-2 break-words font-mono text-xs text-voicebox-secondary">
                        {status.documentRoot ??
                          project?.documentRoot ??
                          "Document root unavailable"}
                      </p>
                      {status.startedAt ? (
                        <p className="mt-2 font-mono text-xs uppercase text-voicebox-secondary">
                          Started {formatDate(status.startedAt)}
                        </p>
                      ) : null}
                    </div>
                    <div className="grid content-start gap-2 text-left md:text-right">
                      <StatusBadge label={status.phpVersion ?? "PHP"} tone="success" />
                      <p className="font-mono text-xs text-voicebox-black">
                        {status.url ?? "URL unavailable"}
                      </p>
                    </div>
                  </article>
                );
              })}
            </div>
          )}
        </section>

        <section className="border-2 border-voicebox-black bg-white p-5">
          <div className="border-b border-voicebox-border pb-4">
            <p className="font-mono text-xs uppercase text-voicebox-secondary">Service Inventory</p>
            <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
              Backend Guarded
            </h2>
          </div>
          <div className="mt-5 grid gap-3">
            {services.map((service) => (
              <div
                className="flex items-start justify-between gap-3 border border-voicebox-border bg-voicebox-surface p-3"
                key={service.id}
              >
                <div>
                  <p className="text-sm font-bold text-voicebox-black">{service.name}</p>
                  <p className="mt-1 text-xs text-voicebox-secondary">{service.statusMessage}</p>
                </div>
                <StatusBadge
                  label={formatServiceStatus(service.status)}
                  tone={getServiceStatusTone(service.status)}
                />
              </div>
            ))}
            {!isLoading && services.length === 0 ? (
              <EmptyState
                title="No services registered"
                description="The service inventory command returned no backend service definitions."
              />
            ) : null}
          </div>
        </section>
      </div>
    </PageShell>
  );
}
