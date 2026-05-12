import { useCallback, useEffect, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { WarningPanel } from "../../../shared/components/feedback/warning-panel";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { EmptyState } from "../../../shared/components/ui/empty-state";
import {
  getProjectPhpVersion,
  installProjectPhpRuntime,
  selectProjectPhpVersion,
} from "../api/project.commands";
import { ProjectPhpVersionSelector } from "../components/project-php-version-selector";
import type { PhpRuntimeInstallProvider, ProjectPhpVersionConfig } from "../types/project.types";

const currentProjectId = "current-project";

function getErrorMessage(error: unknown) {
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = (error as { message?: unknown }).message;

    if (typeof message === "string" && message.trim().length > 0) {
      return message;
    }
  }

  return "Project runtime command failed safely. Check the application logs for details.";
}

const providerLabels: Record<PhpRuntimeInstallProvider, string> = {
  homebrew: "Homebrew",
  scoop: "Scoop",
};

export function ProjectsPage() {
  const [config, setConfig] = useState<ProjectPhpVersionConfig>();
  const [draftVersion, setDraftVersion] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [isInstalling, setIsInstalling] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string>();
  const [noticeMessage, setNoticeMessage] = useState<string>();

  const loadConfig = useCallback(async () => {
    setIsLoading(true);
    setErrorMessage(undefined);

    try {
      const nextConfig = await getProjectPhpVersion(currentProjectId);
      setConfig(nextConfig);
      setDraftVersion(nextConfig.selectedPhpVersion);
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadConfig();
  }, [loadConfig]);

  const selectedOption = config?.availablePhpVersions.find(
    (version) => version.version === draftVersion,
  );

  const handleInstall = useCallback(async () => {
    if (!draftVersion || !selectedOption) {
      return;
    }

    const shouldInstall = window.confirm(
      [
        `${selectedOption.label} is not installed for this project.`,
        selectedOption.lifecycleWarning ?? "Install only from a trusted PHP runtime source.",
        "AxiomPHP will run a package-manager install through the Rust backend using Homebrew on macOS or Scoop on Windows.",
        "No shell command is built by the frontend. Only the resolved package-manager executable is allowed by the backend command policy.",
        "Continue?",
      ].join("\n\n"),
    );

    if (!shouldInstall) {
      return;
    }

    setIsInstalling(true);
    setErrorMessage(undefined);
    setNoticeMessage(undefined);

    try {
      const installResult = await installProjectPhpRuntime(currentProjectId, draftVersion);
      setNoticeMessage(
        `${installResult.statusMessage} Provider: ${providerLabels[installResult.provider]}. Package: ${installResult.packageName}.`,
      );
      await loadConfig();
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    } finally {
      setIsInstalling(false);
    }
  }, [draftVersion, loadConfig, selectedOption]);

  const handleSave = useCallback(async () => {
    if (!draftVersion) {
      return;
    }

    setIsSaving(true);
    setErrorMessage(undefined);
    setNoticeMessage(undefined);

    try {
      const nextConfig = await selectProjectPhpVersion(currentProjectId, draftVersion);
      setConfig(nextConfig);
      setDraftVersion(nextConfig.selectedPhpVersion);
      setNoticeMessage(`PHP ${nextConfig.selectedPhpVersion} binary selected for this project.`);
    } catch (error) {
      setErrorMessage(getErrorMessage(error));
    } finally {
      setIsSaving(false);
    }
  }, [draftVersion]);

  return (
    <PageShell
      title="Projects"
      description="Project runtime preferences and PHP binary installation are controlled by the Rust backend. Project process execution remains disabled."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {noticeMessage ? <WarningPanel message={noticeMessage} /> : null}
      {isLoading ? <LoadingState label="Loading project runtime preference" /> : null}
      {!isLoading && config ? (
        <ProjectPhpVersionSelector
          config={config}
          draftVersion={draftVersion}
          isInstalling={isInstalling}
          isSaving={isSaving}
          onDraftVersionChange={setDraftVersion}
          onInstall={handleInstall}
          onSave={handleSave}
        />
      ) : null}
      {!isLoading && !config ? (
        <EmptyState
          title="No project runtime profile"
          description="The backend did not return a project runtime configuration."
        />
      ) : null}
    </PageShell>
  );
}
