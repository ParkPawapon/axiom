import { useCallback, useEffect, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { StatusPanel } from "../../../shared/components/feedback/status-panel";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import { AppPreferencesForm } from "../components/app-preferences-form";
import { SecuritySettingsForm } from "../components/security-settings-form";
import { SettingsSection } from "../components/settings-section";
import { readSettings } from "../api/settings.commands";
import type { AppSettings } from "../types/settings.types";

export function SettingsPage() {
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isLoading, setIsLoading] = useState(true);
  const [settings, setSettings] = useState<AppSettings>();

  const loadSettings = useCallback(async () => {
    setIsLoading(true);
    setErrorMessage(undefined);

    try {
      setSettings(await readSettings());
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "Settings could not be loaded from the backend."));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadSettings();
  }, [loadSettings]);

  return (
    <PageShell
      title="Settings"
      description="Typed backend settings loaded from the Rust configuration repository."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {isLoading ? <LoadingState label="Loading settings" /> : null}

      <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_24rem]">
        <div className="grid gap-5">
          <SettingsSection
            title="Application Preferences"
            description="Current desktop defaults that are controlled by source configuration and backend storage boundaries."
          >
            <AppPreferencesForm settings={settings} />
          </SettingsSection>

          <SettingsSection
            title="Security Posture"
            description="Controls already enforced by the current Rust and Tauri architecture."
          >
            <SecuritySettingsForm settings={settings} />
          </SettingsSection>
        </div>

        <StatusPanel title="Settings Boundary" tone="neutral">
          Runtime paths, Docker context, ports, and audit preferences now flow through validated
          Rust settings use cases. Secrets remain isolated in secure storage.
        </StatusPanel>
      </div>
    </PageShell>
  );
}
