import { useCallback, useEffect, useMemo, useState } from "react";

import { listProjects } from "../../projects/api/project.commands";
import type { Project } from "../../projects/types/project.types";
import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { WarningPanel } from "../../../shared/components/feedback/warning-panel";
import { Button } from "../../../shared/components/ui/button";
import { Select } from "../../../shared/components/ui/select";
import {
  ensureProjectDockerVolumes,
  generateProjectDockerCompose,
  getDockerDiagnostics,
  getProjectDockerStatus,
  readProjectDockerLogs,
  removeProjectDockerVolumes,
  restartProjectDockerServices,
  startProjectDockerServices,
  stopProjectDockerServices,
} from "../api/docker.commands";
import type {
  DockerComposeProfile,
  DockerDiagnosticsReport,
  DockerProjectComposePlan,
  DockerProjectLogReadResult,
  DockerProjectRuntimeStatus,
} from "../types/docker.types";

type DockerAction =
  | "diagnostics"
  | "generate"
  | "logs"
  | "restart"
  | "start"
  | "status"
  | "stop"
  | "volumes:create"
  | "volumes:remove";

const PROFILE_OPTIONS: ReadonlyArray<{
  label: string;
  profile: DockerComposeProfile;
  description: string;
}> = [
  {
    label: "PHP",
    profile: "php",
    description: "PHP local development server profile",
  },
  {
    label: "MySQL",
    profile: "mysql",
    description: "Project-specific MySQL service and named volume",
  },
  {
    label: "PostgreSQL",
    profile: "postgresql",
    description: "Project-specific PostgreSQL service and named volume",
  },
  {
    label: "Reverse proxy",
    profile: "reverseProxy",
    description: "Project-specific reverse proxy in front of PHP",
  },
];

const DEFAULT_PROFILES = new Set<DockerComposeProfile>(["php"]);

function getErrorMessage(error: unknown) {
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = (error as { message?: unknown }).message;

    if (typeof message === "string" && message.trim().length > 0) {
      return message;
    }
  }

  return "Docker command failed safely. Check the backend logs for details.";
}

function selectedProfiles(profileState: Record<DockerComposeProfile, boolean>) {
  return PROFILE_OPTIONS.map((option) => option.profile).filter((profile) => profileState[profile]);
}

export function ProjectDockerOrchestrationPanel() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProjectId, setSelectedProjectId] = useState("");
  const [profileState, setProfileState] = useState<Record<DockerComposeProfile, boolean>>({
    mysql: false,
    php: true,
    postgresql: false,
    reverseProxy: false,
  });
  const [tailLines, setTailLines] = useState(200);
  const [diagnostics, setDiagnostics] = useState<DockerDiagnosticsReport>();
  const [plan, setPlan] = useState<DockerProjectComposePlan>();
  const [runtime, setRuntime] = useState<DockerProjectRuntimeStatus>();
  const [logs, setLogs] = useState<DockerProjectLogReadResult>();
  const [busyAction, setBusyAction] = useState<DockerAction>();
  const [errorMessage, setErrorMessage] = useState<string>();
  const [noticeMessage, setNoticeMessage] = useState<string>();

  const profiles = useMemo(() => selectedProfiles(profileState), [profileState]);
  const selectedProject = projects.find((project) => project.id === selectedProjectId);
  const canRunProjectCommand = Boolean(selectedProjectId) && !busyAction;

  const loadProjects = useCallback(async () => {
    const loadedProjects = await listProjects();
    setProjects(loadedProjects);
    setSelectedProjectId((currentProjectId) => {
      if (loadedProjects.some((project) => project.id === currentProjectId)) {
        return currentProjectId;
      }

      return loadedProjects[0]?.id ?? "";
    });
  }, []);

  const refreshDiagnostics = useCallback(async () => {
    setDiagnostics(await getDockerDiagnostics());
  }, []);

  useEffect(() => {
    void Promise.all([loadProjects(), refreshDiagnostics()]).catch((error: unknown) => {
      setErrorMessage(getErrorMessage(error));
    });
  }, [loadProjects, refreshDiagnostics]);

  const runAction = useCallback(
    async <T,>(action: DockerAction, command: () => Promise<T>, onSuccess: (result: T) => void) => {
      setBusyAction(action);
      setErrorMessage(undefined);
      setNoticeMessage(undefined);

      try {
        const result = await command();
        onSuccess(result);
      } catch (error) {
        setErrorMessage(getErrorMessage(error));
      } finally {
        setBusyAction(undefined);
      }
    },
    [],
  );

  const toggleProfile = (profile: DockerComposeProfile) => {
    setProfileState((currentState) => {
      const nextState = {
        ...currentState,
        [profile]: !currentState[profile],
      };
      const hasAnyProfile = selectedProfiles(nextState).length > 0;

      if (!hasAnyProfile) {
        return {
          ...nextState,
          php: true,
        };
      }

      return nextState;
    });
  };

  const generateCompose = () => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(
      "generate",
      () => generateProjectDockerCompose(selectedProjectId, profiles),
      (result) => {
        setPlan(result);
        setNoticeMessage(result.statusMessage);
      },
    );
  };

  const refreshStatus = () => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(
      "status",
      () => getProjectDockerStatus(selectedProjectId),
      (result) => {
        setRuntime(result);
        setNoticeMessage(result.statusMessage);
      },
    );
  };

  const startServices = () => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(
      "start",
      () => startProjectDockerServices(selectedProjectId, profiles),
      (result) => {
        setPlan(result.plan);
        setRuntime(result.runtime);
        setNoticeMessage(result.statusMessage);
      },
    );
  };

  const stopServices = () => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(
      "stop",
      () => stopProjectDockerServices(selectedProjectId),
      (result) => {
        setPlan(result.plan);
        setRuntime(result.runtime);
        setNoticeMessage(result.statusMessage);
      },
    );
  };

  const restartServices = () => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(
      "restart",
      () => restartProjectDockerServices(selectedProjectId, profiles),
      (result) => {
        setPlan(result.plan);
        setRuntime(result.runtime);
        setNoticeMessage(result.statusMessage);
      },
    );
  };

  const ensureVolumes = () => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(
      "volumes:create",
      () => ensureProjectDockerVolumes(selectedProjectId, profiles),
      (result) => {
        setNoticeMessage(result.statusMessage);
        void getProjectDockerStatus(selectedProjectId).then(setRuntime);
      },
    );
  };

  const removeVolumes = () => {
    if (!selectedProjectId) {
      return;
    }

    const confirmed = window.confirm(
      "Remove Docker volumes for this project? Database container data in those volumes will be deleted.",
    );

    if (!confirmed) {
      return;
    }

    void runAction(
      "volumes:remove",
      () => removeProjectDockerVolumes(selectedProjectId),
      (result) => {
        setNoticeMessage(result.statusMessage);
        void getProjectDockerStatus(selectedProjectId).then(setRuntime);
      },
    );
  };

  const readLogs = () => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(
      "logs",
      () => readProjectDockerLogs(selectedProjectId, tailLines),
      (result) => {
        setLogs(result);
        setNoticeMessage(result.statusMessage);
      },
    );
  };

  return (
    <section className="flex flex-col gap-4 border border-voicebox-border bg-voicebox-surface p-4">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h2 className="font-display text-2xl uppercase leading-none">
            Project Docker Orchestration
          </h2>
          <p className="mt-2 max-w-3xl text-sm text-voicebox-secondary">
            Per-project Compose files, service profiles, volumes, diagnostics, and sanitized logs
            are mediated by Rust backend commands.
          </p>
        </div>
        <Button
          disabled={Boolean(busyAction)}
          onClick={() =>
            void runAction("diagnostics", getDockerDiagnostics, (result) => {
              setDiagnostics(result);
              setNoticeMessage(result.statusMessage);
            })
          }
          variant="secondary"
        >
          Docker Diagnostics
        </Button>
      </div>

      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {noticeMessage ? <WarningPanel message={noticeMessage} /> : null}

      <div className="grid gap-4 lg:grid-cols-[minmax(0,0.8fr)_minmax(0,1.2fr)]">
        <div className="flex flex-col gap-4">
          <label className="flex flex-col gap-2 text-xs font-semibold uppercase tracking-wide">
            Project
            <Select
              disabled={projects.length === 0 || Boolean(busyAction)}
              onChange={(event) => setSelectedProjectId(event.target.value)}
              value={selectedProjectId}
            >
              {projects.length === 0 ? <option value="">No projects available</option> : null}
              {projects.map((project) => (
                <option key={project.id} value={project.id}>
                  {project.name}
                </option>
              ))}
            </Select>
          </label>

          {selectedProject ? (
            <div className="border border-voicebox-border bg-white p-3 text-xs text-voicebox-secondary">
              <p className="font-mono uppercase text-voicebox-primary">{selectedProject.id}</p>
              <p className="mt-2 break-all">{selectedProject.documentRoot}</p>
            </div>
          ) : null}

          <div className="grid gap-2">
            {PROFILE_OPTIONS.map((option) => (
              <label
                className="flex items-start gap-3 border border-voicebox-border bg-white p-3 text-sm"
                key={option.profile}
              >
                <input
                  checked={profileState[option.profile] || DEFAULT_PROFILES.has(option.profile)}
                  className="mt-1 h-4 w-4 accent-voicebox-red"
                  disabled={DEFAULT_PROFILES.has(option.profile) || Boolean(busyAction)}
                  onChange={() => toggleProfile(option.profile)}
                  type="checkbox"
                />
                <span>
                  <span className="block font-semibold text-voicebox-primary">{option.label}</span>
                  <span className="block text-xs text-voicebox-secondary">
                    {option.description}
                  </span>
                </span>
              </label>
            ))}
          </div>

          <label className="flex flex-col gap-2 text-xs font-semibold uppercase tracking-wide">
            Log tail
            <Select
              disabled={Boolean(busyAction)}
              onChange={(event) => setTailLines(Number(event.target.value))}
              value={tailLines}
            >
              <option value={100}>100 lines</option>
              <option value={200}>200 lines</option>
              <option value={500}>500 lines</option>
              <option value={1000}>1000 lines</option>
            </Select>
          </label>
        </div>

        <div className="grid gap-3">
          <div className="flex flex-wrap gap-2">
            <Button disabled={!canRunProjectCommand} onClick={generateCompose} variant="secondary">
              Generate Compose
            </Button>
            <Button disabled={!canRunProjectCommand} onClick={refreshStatus} variant="secondary">
              Status
            </Button>
            <Button disabled={!canRunProjectCommand} onClick={startServices}>
              Start
            </Button>
            <Button disabled={!canRunProjectCommand} onClick={stopServices} variant="secondary">
              Stop
            </Button>
            <Button disabled={!canRunProjectCommand} onClick={restartServices} variant="secondary">
              Restart
            </Button>
            <Button disabled={!canRunProjectCommand} onClick={ensureVolumes} variant="secondary">
              Ensure Volumes
            </Button>
            <Button disabled={!canRunProjectCommand} onClick={removeVolumes} variant="secondary">
              Remove Volumes
            </Button>
            <Button disabled={!canRunProjectCommand} onClick={readLogs} variant="secondary">
              Read Logs
            </Button>
          </div>

          {diagnostics ? (
            <div className="border border-voicebox-border bg-white p-3">
              <p className="font-mono text-xs uppercase text-voicebox-secondary">
                {diagnostics.statusMessage}
              </p>
              <div className="mt-3 grid gap-2">
                {diagnostics.checks.map((check) => (
                  <div className="flex items-start justify-between gap-3 text-sm" key={check.name}>
                    <span className="font-semibold">{check.name}</span>
                    <span className={check.healthy ? "text-voicebox-success" : "text-voicebox-red"}>
                      {check.statusMessage}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          ) : null}

          {plan ? (
            <div className="border border-voicebox-border bg-white p-3 text-sm">
              <div className="flex flex-wrap justify-between gap-3">
                <p className="font-semibold">{plan.composeProjectName}</p>
                <p className="font-mono text-xs uppercase">
                  {plan.composeFileWritten ? "compose written" : "trust blocked"}
                </p>
              </div>
              <p className="mt-2 break-all text-xs text-voicebox-secondary">
                {plan.composeFilePath}
              </p>
              <div className="mt-3 grid gap-2">
                {plan.imageTrust.map((trust) => (
                  <div
                    className="border border-voicebox-border bg-voicebox-surface p-2 text-xs"
                    key={`${trust.profile}-${trust.image}`}
                  >
                    <p className="font-mono uppercase">{trust.profile}</p>
                    <p className="mt-1 break-all">{trust.image}</p>
                    <p className={trust.allowed ? "text-voicebox-success" : "text-voicebox-red"}>
                      {trust.statusMessage}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          ) : null}

          {runtime ? (
            <div className="border border-voicebox-border bg-white p-3 text-sm">
              <div className="flex flex-wrap justify-between gap-3">
                <p className="font-semibold">{runtime.statusMessage}</p>
                <p className="font-mono text-xs uppercase">
                  {runtime.engineRunning ? "engine ready" : "engine unavailable"}
                </p>
              </div>
              <div className="mt-3 grid gap-2">
                {runtime.containers.length === 0 ? (
                  <p className="text-xs text-voicebox-secondary">No project containers reported.</p>
                ) : (
                  runtime.containers.map((container) => (
                    <div
                      className="grid gap-1 border border-voicebox-border bg-voicebox-surface p-2 text-xs"
                      key={container.name}
                    >
                      <p className="font-mono uppercase">{container.serviceName}</p>
                      <p>{container.name}</p>
                      <p>{container.status}</p>
                    </div>
                  ))
                )}
                {runtime.volumes.map((volume) => (
                  <div
                    className="grid gap-1 border border-voicebox-border bg-voicebox-surface p-2 text-xs"
                    key={volume.name}
                  >
                    <p className="font-mono uppercase">{volume.serviceName}</p>
                    <p>{volume.name}</p>
                    <p>{volume.created ? "Volume exists" : "Volume missing"}</p>
                  </div>
                ))}
              </div>
            </div>
          ) : null}

          {logs ? (
            <pre className="max-h-80 overflow-auto border border-voicebox-border bg-voicebox-black p-3 font-mono text-xs text-white">
              {logs.lines.length > 0 ? logs.lines.join("\n") : "No Docker logs returned."}
            </pre>
          ) : null}
        </div>
      </div>
    </section>
  );
}
