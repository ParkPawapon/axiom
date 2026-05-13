import { confirm } from "@tauri-apps/plugin-dialog";
import { useMemo, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { WarningPanel } from "../../../shared/components/feedback/warning-panel";
import { Button } from "../../../shared/components/ui/button";
import { Chip } from "../../../shared/components/ui/chip";
import { Input } from "../../../shared/components/ui/input";
import { Select } from "../../../shared/components/ui/select";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import {
  createProject,
  getProjectPhpVersion,
  installProjectPhpRuntime,
  selectProjectPhpVersion,
  startProjectPhpProcess,
  validateProjectPath,
} from "../api/project.commands";
import type {
  Project,
  ProjectDraft,
  ProjectPhpProcessStatus,
  ProjectPhpVersionConfig,
} from "../types/project.types";
import { ProjectPathPicker } from "./project-path-picker";

type WizardStep = "details" | "runtime" | "process" | "done";

interface ProjectSetupWizardProps {
  onProjectReady: (projectId: string) => Promise<void>;
}

const emptyDraft: ProjectDraft = {
  documentRoot: "",
  name: "",
};

const stepLabels: Record<WizardStep, string> = {
  details: "Project",
  done: "Ready",
  process: "Process",
  runtime: "Runtime",
};

export function ProjectSetupWizard({ onProjectReady }: ProjectSetupWizardProps) {
  const [createdProject, setCreatedProject] = useState<Project>();
  const [draft, setDraft] = useState<ProjectDraft>(emptyDraft);
  const [draftVersion, setDraftVersion] = useState("");
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isBusy, setIsBusy] = useState(false);
  const [noticeMessage, setNoticeMessage] = useState<string>();
  const [processStatus, setProcessStatus] = useState<ProjectPhpProcessStatus>();
  const [runtimeConfig, setRuntimeConfig] = useState<ProjectPhpVersionConfig>();
  const [step, setStep] = useState<WizardStep>("details");

  const selectedOption = useMemo(
    () => runtimeConfig?.availablePhpVersions.find((version) => version.version === draftVersion),
    [draftVersion, runtimeConfig],
  );

  function resetWizard() {
    setCreatedProject(undefined);
    setDraft(emptyDraft);
    setDraftVersion("");
    setErrorMessage(undefined);
    setNoticeMessage(undefined);
    setProcessStatus(undefined);
    setRuntimeConfig(undefined);
    setStep("details");
  }

  async function handleCreateProject() {
    setIsBusy(true);
    setErrorMessage(undefined);
    setNoticeMessage(undefined);

    try {
      await validateProjectPath(draft.documentRoot);
      const project = await createProject(draft.name, draft.documentRoot);
      const config = await getProjectPhpVersion(project.id);

      setCreatedProject(project);
      setRuntimeConfig(config);
      setDraftVersion(config.selectedPhpVersion);
      setStep("runtime");
      setNoticeMessage(`${project.name} was created and persisted.`);
      await onProjectReady(project.id);
    } catch (error) {
      setErrorMessage(
        getErrorMessage(error, "Project setup failed safely before runtime configuration."),
      );
    } finally {
      setIsBusy(false);
    }
  }

  async function handleInstallRuntime() {
    if (!createdProject || !selectedOption) {
      return;
    }

    const shouldInstall = await confirm(
      [
        `${selectedOption.label} is not installed for ${createdProject.name}.`,
        selectedOption.lifecycleWarning ?? "Install only from a trusted PHP runtime source.",
        "AxiomPHP will ask the Rust backend to use its allowlisted package-manager adapter.",
        "Continue?",
      ].join("\n\n"),
      { kind: "warning", title: "Install PHP runtime" },
    );

    if (!shouldInstall) {
      return;
    }

    setIsBusy(true);
    setErrorMessage(undefined);
    setNoticeMessage(undefined);

    try {
      const result = await installProjectPhpRuntime(createdProject.id, draftVersion);
      const config = await getProjectPhpVersion(createdProject.id);

      setRuntimeConfig(config);
      setDraftVersion(config.selectedPhpVersion);
      setNoticeMessage(`${result.statusMessage} Package: ${result.packageName}.`);
      await onProjectReady(createdProject.id);
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "PHP runtime installation failed safely."));
    } finally {
      setIsBusy(false);
    }
  }

  async function handleSelectRuntime() {
    if (!createdProject || !draftVersion) {
      return;
    }

    setIsBusy(true);
    setErrorMessage(undefined);
    setNoticeMessage(undefined);

    try {
      const config = await selectProjectPhpVersion(createdProject.id, draftVersion);
      setRuntimeConfig(config);
      setDraftVersion(config.selectedPhpVersion);
      setStep("process");
      setNoticeMessage(`PHP ${config.selectedPhpVersion} selected for ${createdProject.name}.`);
      await onProjectReady(createdProject.id);
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "PHP runtime selection failed safely."));
    } finally {
      setIsBusy(false);
    }
  }

  async function handleStartProcess() {
    if (!createdProject) {
      return;
    }

    const shouldStart = await confirm(
      [
        `Start ${createdProject.name} now?`,
        "The backend will bind the PHP built-in server to 127.0.0.1 and use this project's document root.",
        "No shell command is built by the frontend.",
      ].join("\n\n"),
      { kind: "info", title: "Start project process" },
    );

    if (!shouldStart) {
      return;
    }

    setIsBusy(true);
    setErrorMessage(undefined);
    setNoticeMessage(undefined);

    try {
      const status = await startProjectPhpProcess(createdProject.id);
      setProcessStatus(status);
      setStep("done");
      setNoticeMessage(status.statusMessage);
      await onProjectReady(createdProject.id);
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "Project process start failed safely."));
    } finally {
      setIsBusy(false);
    }
  }

  const canCreate = draft.name.trim().length > 0 && draft.documentRoot.trim().length > 0;
  const canSelectRuntime = Boolean(selectedOption?.installed && selectedOption.canSwitch);
  const canInstallRuntime = Boolean(selectedOption && !selectedOption.installed);

  return (
    <section className="border-2 border-voicebox-black bg-white p-5">
      <div className="border-b border-voicebox-border pb-4">
        <p className="font-mono text-xs uppercase text-voicebox-secondary">Setup Wizard</p>
        <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
          New PHP Project
        </h2>
        <div className="mt-4 flex flex-wrap gap-2">
          {(Object.keys(stepLabels) as WizardStep[]).map((wizardStep) => (
            <Chip key={wizardStep} tone={wizardStep === step ? "neutral" : "warning"}>
              {stepLabels[wizardStep]}
            </Chip>
          ))}
        </div>
      </div>

      <div className="mt-5 grid gap-4">
        {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
        {noticeMessage ? <WarningPanel message={noticeMessage} /> : null}

        {step === "details" ? (
          <>
            <label className="grid gap-2">
              <span className="text-sm font-bold text-voicebox-black">Project name</span>
              <Input
                disabled={isBusy}
                placeholder="Local app"
                value={draft.name}
                onChange={(event) =>
                  setDraft((currentDraft) => ({
                    ...currentDraft,
                    name: event.currentTarget.value,
                  }))
                }
              />
            </label>
            <ProjectPathPicker
              disabled={isBusy}
              label="Document root"
              value={draft.documentRoot}
              onChange={(documentRoot) =>
                setDraft((currentDraft) => ({ ...currentDraft, documentRoot }))
              }
            />
            <Button disabled={!canCreate || isBusy} onClick={() => void handleCreateProject()}>
              {isBusy ? "Creating" : "Validate & Create"}
            </Button>
          </>
        ) : null}

        {step === "runtime" && runtimeConfig ? (
          <>
            <label className="grid gap-2">
              <span className="text-sm font-bold text-voicebox-black">PHP version</span>
              <Select
                disabled={isBusy}
                value={draftVersion}
                onChange={(event) => setDraftVersion(event.currentTarget.value)}
              >
                {runtimeConfig.availablePhpVersions.map((version) => (
                  <option key={version.version} value={version.version}>
                    {version.label}
                    {version.installed ? " / Installed" : " / Requires install"}
                  </option>
                ))}
              </Select>
            </label>
            {selectedOption?.lifecycleWarning ? (
              <p className="border-l-2 border-voicebox-red pl-3 text-xs leading-relaxed text-voicebox-red">
                {selectedOption.lifecycleWarning}
              </p>
            ) : null}
            <div className="flex flex-wrap gap-2">
              <Button
                disabled={!canInstallRuntime || isBusy}
                onClick={() => void handleInstallRuntime()}
                variant="secondary"
              >
                {isBusy ? "Installing" : "Install Runtime"}
              </Button>
              <Button
                disabled={!canSelectRuntime || isBusy}
                onClick={() => void handleSelectRuntime()}
              >
                {isBusy ? "Selecting" : "Select Runtime"}
              </Button>
            </div>
          </>
        ) : null}

        {step === "process" && createdProject ? (
          <>
            <div className="border border-voicebox-border bg-voicebox-surface p-3">
              <p className="font-mono text-xs uppercase text-voicebox-secondary">Ready to start</p>
              <p className="mt-2 text-sm text-voicebox-secondary">
                {createdProject.name} has a persisted project profile and selected PHP runtime.
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              <Button disabled={isBusy} onClick={() => void handleStartProcess()}>
                {isBusy ? "Starting" : "Start Now"}
              </Button>
              <Button disabled={isBusy} onClick={() => setStep("done")} variant="secondary">
                Finish Without Starting
              </Button>
            </div>
          </>
        ) : null}

        {step === "done" && createdProject ? (
          <>
            <div className="border border-voicebox-border bg-voicebox-surface p-3">
              <p className="font-mono text-xs uppercase text-voicebox-secondary">Project ready</p>
              <p className="mt-2 font-display text-xl uppercase leading-none text-voicebox-black">
                {createdProject.name}
              </p>
              <p className="mt-2 break-words font-mono text-xs text-voicebox-secondary">
                {createdProject.documentRoot}
              </p>
              {processStatus?.url ? (
                <p className="mt-2 font-mono text-xs text-voicebox-success">
                  Running at {processStatus.url}
                </p>
              ) : null}
            </div>
            <Button disabled={isBusy} onClick={resetWizard} variant="secondary">
              Set Up Another Project
            </Button>
          </>
        ) : null}
      </div>
    </section>
  );
}
