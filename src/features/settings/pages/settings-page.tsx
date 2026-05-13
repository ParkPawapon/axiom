import { PageShell } from "../../../shared/components/layout/page-shell";
import { StatusPanel } from "../../../shared/components/feedback/status-panel";
import { AppPreferencesForm } from "../components/app-preferences-form";
import { SecuritySettingsForm } from "../components/security-settings-form";
import { SettingsSection } from "../components/settings-section";

export function SettingsPage() {
  return (
    <PageShell
      title="Settings"
      description="Read-only frontend settings surface for the current build. Editable persistence will be enabled only after typed backend settings commands are available."
    >
      <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_24rem]">
        <div className="grid gap-5">
          <SettingsSection
            title="Application Preferences"
            description="Current desktop defaults that are controlled by source configuration and backend storage boundaries."
          >
            <AppPreferencesForm />
          </SettingsSection>

          <SettingsSection
            title="Security Posture"
            description="Controls already enforced by the current Rust and Tauri architecture."
          >
            <SecuritySettingsForm />
          </SettingsSection>
        </div>

        <StatusPanel title="Not Editable Yet" tone="warning">
          Runtime paths, Docker preferences, ports, certificates, and database credentials remain
          non-editable until the backend exposes validated settings use cases and secure storage.
        </StatusPanel>
      </div>
    </PageShell>
  );
}
