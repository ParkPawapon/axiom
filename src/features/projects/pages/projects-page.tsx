import { useCallback, useEffect, useMemo, useState } from "react";
import { confirm } from "@tauri-apps/plugin-dialog";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { WarningPanel } from "../../../shared/components/feedback/warning-panel";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { Button } from "../../../shared/components/ui/button";
import { EmptyState } from "../../../shared/components/ui/empty-state";
import {
  deleteProject,
  getProjectPhpProcessStatus,
  getProjectPhpVersion,
  installProjectPhpRuntime,
  listProjects,
  restartProjectPhpProcess,
  restartProjectPhpProcesses,
  selectProjectPhpVersion,
  startProjectPhpProcess,
  startProjectPhpProcesses,
  stopProjectPhpProcess,
  stopProjectPhpProcesses,
  updateProject,
} from "../api/project.commands";
import { ProjectList } from "../components/project-list";
import { ProjectPhpProcessPanel } from "../components/project-php-process-panel";
import { ProjectPhpVersionSelector } from "../components/project-php-version-selector";
import { ProjectSetupWizard } from "../components/project-setup-wizard";
import type {
  Project,
  ProjectDraft,
  ProjectPhpProcessActionResult,
  ProjectPhpProcessStatus,
  ProjectPhpVersionConfig,
} from "../types/project.types";
import { formatPhpInstallResult } from "../utils/format-php-install-result";

function getErrorMessage(error: unknown) {
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = (error as { message?: unknown }).message;

    if (typeof message === "string" && message.trim().length > 0) {
      return message;
    }
  }

  return "Project command failed safely. Check the application logs for details.";
}

export function ProjectsPage() {
  const [config, setConfig] = useState<ProjectPhpVersionConfig>();
  const [draftVersion, setDraftVersion] = useState("");
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isInstalling, setIsInstalling] = useState(false);
  const [isProcessBusy, setIsProcessBusy] = useState(false);
  const [isProjectBusy, setIsProjectBusy] = useState(false);
  const [isProjectsLoading, setIsProjectsLoading] = useState(true);
  const [isRuntimeLoading, setIsRuntimeLoading] = useState(false);
  const [isSavingRuntime, setIsSavingRuntime] = useState(false);
  const [noticeMessage, setNoticeMessage] = useState<string>();
  const [processStatus, setProcessStatus] = useState<ProjectPhpProcessStatus>();
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedActionProjectIds, setSelectedActionProjectIds] = useState<string[]>([]);
  const [selectedProjectId, setSelectedProjectId] = useState<string>();

  const selectedProject = useMemo(
    () => projects.find((project) => project.id === selectedProjectId),
    [projects, selectedProjectId],
  );

  const loadProjects = useCallback(async (preferredProjectId?: string) => {
    setIsProjectsLoading(true);
    setErrorMessage(undefined);

    try {
      const nextProjects = await listProjects();
      setProjects(nextProjects);
      setSelectedActionProjectIds((currentIds) =>
        currentIds.filter((projectId) => nextProjects.some((project) => project.id === projectId)),
      );
      setSelectedProjectId((currentProjectId) => {
        const nextSelectedProjectId = preferredProjectId ?? currentProjectId ?? nextProjects[0]?.id;

        if (
          nextSelectedProjectId &&
          nextProjects.some((project) => project.id === nextSelectedProjectId)
        ) {
          return nextSelectedProjectId;
        }

        return nextProjects[0]?.id;
      });
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    } finally {
      setIsProjectsLoading(false);
    }
  }, []);

  const loadConfig = useCallback(async (projectId: string) => {
    setIsRuntimeLoading(true);
    setErrorMessage(undefined);

    try {
      const nextConfig = await getProjectPhpVersion(projectId);
      setConfig(nextConfig);
      setDraftVersion(nextConfig.selectedPhpVersion);
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    } finally {
      setIsRuntimeLoading(false);
    }
  }, []);

  const loadProcessStatus = useCallback(async (projectId: string) => {
    try {
      const status = await getProjectPhpProcessStatus(projectId);
      setProcessStatus(status);
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    }
  }, []);

  useEffect(() => {
    void loadProjects();
  }, [loadProjects]);

  useEffect(() => {
    if (!selectedProjectId) {
      setConfig(undefined);
      setDraftVersion("");
      setProcessStatus(undefined);
      return;
    }

    void loadConfig(selectedProjectId);
    void loadProcessStatus(selectedProjectId);
  }, [loadConfig, loadProcessStatus, selectedProjectId]);

  const selectedActionProjects = useMemo(
    () => projects.filter((project) => selectedActionProjectIds.includes(project.id)),
    [projects, selectedActionProjectIds],
  );

  const selectedOption = config?.availablePhpVersions.find(
    (version) => version.version === draftVersion,
  );

  const handleUpdateProject = useCallback(
    async (projectId: string, draft: ProjectDraft) => {
      setIsProjectBusy(true);
      setErrorMessage(undefined);
      setNoticeMessage(undefined);

      try {
        const project = await updateProject(projectId, draft.name, draft.documentRoot);
        await loadProjects(project.id);
        setNoticeMessage(`${project.name} was updated.`);
      } catch (error) {
        setErrorMessage(getErrorMessage(error));
      } finally {
        setIsProjectBusy(false);
      }
    },
    [loadProjects],
  );

  const handleDeleteProject = useCallback(
    async (projectId: string) => {
      const project = projects.find((currentProject) => currentProject.id === projectId);
      const shouldDelete = await confirm(
        `Delete ${project?.name ?? "this project"} from AxiomPHP configuration? This does not delete files from disk.`,
        { kind: "warning", title: "Delete project profile" },
      );

      if (!shouldDelete) {
        return;
      }

      setIsProjectBusy(true);
      setErrorMessage(undefined);
      setNoticeMessage(undefined);

      try {
        await deleteProject(projectId);
        setNoticeMessage(`${project?.name ?? "Project"} was removed from configuration.`);
        await loadProjects(selectedProjectId === projectId ? undefined : selectedProjectId);
      } catch (error) {
        setErrorMessage(getErrorMessage(error));
      } finally {
        setIsProjectBusy(false);
      }
    },
    [loadProjects, projects, selectedProjectId],
  );

  const handleInstall = useCallback(async () => {
    if (!draftVersion || !selectedOption || !selectedProjectId) {
      return;
    }

    const shouldInstall = await confirm(
      [
        `${selectedOption.label} is not installed for this project.`,
        selectedOption.lifecycleWarning ?? "Install only from a trusted PHP runtime source.",
        "AxiomPHP will run a package-manager install through the Rust backend using Homebrew on macOS or Scoop on Windows.",
        "No shell command is built by the frontend. Only the resolved package-manager executable is allowed by the backend command policy.",
        "Continue?",
      ].join("\n\n"),
      { kind: "warning", title: "Install PHP runtime" },
    );

    if (!shouldInstall) {
      return;
    }

    setIsInstalling(true);
    setErrorMessage(undefined);
    setNoticeMessage(undefined);

    try {
      const installResult = await installProjectPhpRuntime(selectedProjectId, draftVersion);
      setNoticeMessage(formatPhpInstallResult(installResult));
      await loadConfig(selectedProjectId);
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    } finally {
      setIsInstalling(false);
    }
  }, [draftVersion, loadConfig, selectedOption, selectedProjectId]);

  const handleSaveRuntime = useCallback(async () => {
    if (!draftVersion || !selectedProjectId) {
      return;
    }

    setIsSavingRuntime(true);
    setErrorMessage(undefined);
    setNoticeMessage(undefined);

    try {
      const nextConfig = await selectProjectPhpVersion(selectedProjectId, draftVersion);
      setConfig(nextConfig);
      setDraftVersion(nextConfig.selectedPhpVersion);
      setNoticeMessage(`PHP ${nextConfig.selectedPhpVersion} binary selected for this project.`);
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    } finally {
      setIsSavingRuntime(false);
    }
  }, [draftVersion, selectedProjectId]);

  const handleToggleActionSelection = useCallback((projectId: string) => {
    setSelectedActionProjectIds((currentIds) =>
      currentIds.includes(projectId)
        ? currentIds.filter((currentId) => currentId !== projectId)
        : [...currentIds, projectId],
    );
  }, []);

  const summarizeBatchResults = useCallback(
    (actionLabel: string, results: ProjectPhpProcessActionResult[]) => {
      const succeededCount = results.filter((result) => result.succeeded).length;
      const failedResults = results.filter((result) => !result.succeeded);

      setNoticeMessage(
        `${actionLabel} completed for ${succeededCount}/${results.length} selected projects.`,
      );

      if (failedResults.length > 0) {
        setErrorMessage(
          failedResults
            .map((result) => {
              const projectName =
                projects.find((project) => project.id === result.projectId)?.name ??
                result.projectId;

              return `${projectName}: ${result.errorMessage ?? "Process action failed."}`;
            })
            .join(" "),
        );
      }
    },
    [projects],
  );

  const handleStartProcess = useCallback(async () => {
    if (!selectedProjectId) {
      return;
    }

    const shouldStart = await confirm(
      [
        "Start the selected PHP binary as a local project process?",
        "AxiomPHP will bind the PHP built-in server to 127.0.0.1 only and serve the selected project's document root.",
        "No shell command is built by the frontend. Continue?",
      ].join("\n\n"),
      { kind: "info", title: "Start project process" },
    );

    if (!shouldStart) {
      return;
    }

    setIsProcessBusy(true);
    setErrorMessage(undefined);
    setNoticeMessage(undefined);

    try {
      const status = await startProjectPhpProcess(selectedProjectId);
      setProcessStatus(status);
      setNoticeMessage(status.statusMessage);
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
      await loadProcessStatus(selectedProjectId);
    } finally {
      setIsProcessBusy(false);
    }
  }, [loadProcessStatus, selectedProjectId]);

  const handleStopProcess = useCallback(async () => {
    if (!selectedProjectId) {
      return;
    }

    setIsProcessBusy(true);
    setErrorMessage(undefined);
    setNoticeMessage(undefined);

    try {
      const status = await stopProjectPhpProcess(selectedProjectId);
      setProcessStatus(status);
      setNoticeMessage(status.statusMessage);
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
      await loadProcessStatus(selectedProjectId);
    } finally {
      setIsProcessBusy(false);
    }
  }, [loadProcessStatus, selectedProjectId]);

  const handleRestartProcess = useCallback(async () => {
    if (!selectedProjectId) {
      return;
    }

    const shouldRestart = await confirm(
      [
        "Restart the selected PHP project process?",
        "AxiomPHP will stop the selected project process if it is running, then start it again with its persisted PHP binary and document root.",
        "No shell command is built by the frontend. Continue?",
      ].join("\n\n"),
      { kind: "warning", title: "Restart project process" },
    );

    if (!shouldRestart) {
      return;
    }

    setIsProcessBusy(true);
    setErrorMessage(undefined);
    setNoticeMessage(undefined);

    try {
      const status = await restartProjectPhpProcess(selectedProjectId);
      setProcessStatus(status);
      setNoticeMessage(status.statusMessage);
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
      await loadProcessStatus(selectedProjectId);
    } finally {
      setIsProcessBusy(false);
    }
  }, [loadProcessStatus, selectedProjectId]);

  const handleBatchProcessAction = useCallback(
    async (action: "start" | "stop" | "restart") => {
      if (selectedActionProjectIds.length === 0) {
        return;
      }

      const actionLabel = action === "start" ? "Start" : action === "stop" ? "Stop" : "Restart";
      const shouldRun = await confirm(
        [
          `${actionLabel} PHP project processes for ${selectedActionProjectIds.length} selected projects?`,
          "AxiomPHP runs each project action through the Rust backend. Each project is isolated and same-project actions are guarded.",
          "No shell command is built by the frontend. Continue?",
        ].join("\n\n"),
        { kind: "warning", title: `${actionLabel} selected project processes` },
      );

      if (!shouldRun) {
        return;
      }

      setIsProcessBusy(true);
      setErrorMessage(undefined);
      setNoticeMessage(undefined);

      try {
        const results =
          action === "start"
            ? await startProjectPhpProcesses(selectedActionProjectIds)
            : action === "stop"
              ? await stopProjectPhpProcesses(selectedActionProjectIds)
              : await restartProjectPhpProcesses(selectedActionProjectIds);

        summarizeBatchResults(actionLabel, results);

        if (selectedProjectId) {
          await loadProcessStatus(selectedProjectId);
        }
      } catch (error) {
        setErrorMessage(getErrorMessage(error));
      } finally {
        setIsProcessBusy(false);
      }
    },
    [loadProcessStatus, selectedActionProjectIds, selectedProjectId, summarizeBatchResults],
  );

  return (
    <PageShell
      title="Projects"
      description="Project configuration is persisted by the Rust backend. Runtime controls apply to the selected project profile."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {noticeMessage ? <WarningPanel message={noticeMessage} /> : null}

      <div className="grid gap-5 xl:grid-cols-[minmax(18rem,24rem)_minmax(0,1fr)]">
        <div className="grid content-start gap-5">
          <ProjectSetupWizard onProjectReady={loadProjects} />

          {isProjectsLoading ? <LoadingState label="Loading projects" /> : null}
          {!isProjectsLoading && projects.length === 0 ? (
            <EmptyState
              title="No projects configured"
              description="Add a PHP project and select its document root to create a persisted project profile."
            />
          ) : null}
          {projects.length > 0 ? (
            <ProjectList
              activeProjectId={selectedProjectId}
              isBusy={isProjectBusy}
              projects={projects}
              selectedActionProjectIds={selectedActionProjectIds}
              onDelete={handleDeleteProject}
              onSelect={setSelectedProjectId}
              onToggleActionSelection={handleToggleActionSelection}
              onUpdate={handleUpdateProject}
            />
          ) : null}
        </div>

        <div className="grid content-start gap-5">
          {selectedProject ? (
            <section className="border-2 border-voicebox-black bg-white p-5">
              <p className="font-mono text-xs uppercase text-voicebox-secondary">
                Selected Project
              </p>
              <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
                {selectedProject.name}
              </h2>
              <p className="mt-3 break-words font-mono text-xs text-voicebox-secondary">
                {selectedProject.documentRoot}
              </p>
            </section>
          ) : null}

          {projects.length > 0 ? (
            <section className="border-2 border-voicebox-black bg-white p-5">
              <div className="border-b border-voicebox-border pb-4">
                <p className="font-mono text-xs uppercase text-voicebox-secondary">
                  Multi-project Processes
                </p>
                <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
                  {selectedActionProjects.length} Selected
                </h2>
              </div>
              <div className="mt-5 flex flex-wrap gap-2">
                <Button
                  disabled={selectedActionProjectIds.length === 0 || isProcessBusy}
                  onClick={() => void handleBatchProcessAction("start")}
                >
                  Start Selected
                </Button>
                <Button
                  disabled={selectedActionProjectIds.length === 0 || isProcessBusy}
                  onClick={() => void handleBatchProcessAction("stop")}
                  variant="secondary"
                >
                  Stop Selected
                </Button>
                <Button
                  disabled={selectedActionProjectIds.length === 0 || isProcessBusy}
                  onClick={() => void handleBatchProcessAction("restart")}
                  variant="secondary"
                >
                  Restart Selected
                </Button>
              </div>
            </section>
          ) : null}

          {isRuntimeLoading ? <LoadingState label="Loading project runtime preference" /> : null}
          {!isRuntimeLoading && selectedProject && config ? (
            <>
              <ProjectPhpVersionSelector
                config={config}
                draftVersion={draftVersion}
                isInstalling={isInstalling}
                isSaving={isSavingRuntime}
                onDraftVersionChange={setDraftVersion}
                onInstall={handleInstall}
                onSave={handleSaveRuntime}
              />
              <ProjectPhpProcessPanel
                canStart={Boolean(config.selectedPhpBinary)}
                isBusy={isProcessBusy}
                status={processStatus}
                onRefresh={() => void loadProcessStatus(selectedProject.id)}
                onRestart={handleRestartProcess}
                onStart={handleStartProcess}
                onStop={handleStopProcess}
              />
            </>
          ) : null}
        </div>
      </div>
    </PageShell>
  );
}
