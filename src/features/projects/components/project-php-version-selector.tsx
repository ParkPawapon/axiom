import { Button } from "../../../shared/components/ui/button";
import { Select } from "../../../shared/components/ui/select";
import type { PhpVersionSupportPhase, ProjectPhpVersionConfig } from "../types/project.types";

interface ProjectPhpVersionSelectorProps {
  config: ProjectPhpVersionConfig;
  draftVersion: string;
  isSaving: boolean;
  isInstalling: boolean;
  onDraftVersionChange: (version: string) => void;
  onInstall: () => void;
  onSave: () => void;
}

const supportPhaseLabels: Record<PhpVersionSupportPhase, string> = {
  active: "ACTIVE SUPPORT",
  endOfLife: "END OF LIFE",
  security: "SECURITY SUPPORT",
};

export function ProjectPhpVersionSelector({
  config,
  draftVersion,
  isInstalling,
  isSaving,
  onDraftVersionChange,
  onInstall,
  onSave,
}: ProjectPhpVersionSelectorProps) {
  const selectedOption = config.availablePhpVersions.find(
    (version) => version.version === draftVersion,
  );
  const hasChanges = draftVersion !== config.selectedPhpVersion;
  const isBusy = isSaving || isInstalling;
  const canSwitch = Boolean(selectedOption?.canSwitch && (hasChanges || !config.selectedPhpBinary));
  const canInstall = Boolean(selectedOption && !selectedOption.installed);

  return (
    <section className="border-2 border-voicebox-black bg-white p-5">
      <div className="flex flex-col gap-2 border-b border-voicebox-border pb-4 md:flex-row md:items-end md:justify-between">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">Project Runtime</p>
          <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
            PHP Version
          </h2>
        </div>
        <p className="font-mono text-xs uppercase text-voicebox-secondary">
          Project: {config.projectId}
        </p>
      </div>

      <div className="mt-5 grid gap-4 lg:grid-cols-[minmax(0,1fr)_auto]">
        <label className="grid gap-2">
          <span className="text-sm font-bold text-voicebox-black">Target PHP branch</span>
          <Select
            aria-label="Target PHP branch"
            disabled={isBusy}
            value={draftVersion}
            onChange={(event) => onDraftVersionChange(event.currentTarget.value)}
          >
            {config.availablePhpVersions.map((version) => (
              <option key={version.version} value={version.version}>
                {version.label}
                {version.recommended ? " (Recommended)" : ""}
              </option>
            ))}
          </Select>
        </label>

        <div className="flex flex-wrap items-end gap-2 self-end">
          <Button disabled={!canInstall || isBusy} onClick={onInstall} variant="secondary">
            {isInstalling ? "Preparing" : "Install"}
          </Button>
          <Button disabled={!canSwitch || isBusy} onClick={onSave} variant="primary">
            {isSaving ? "Switching" : "Switch"}
          </Button>
        </div>
      </div>

      <div className="mt-5 grid gap-2 md:grid-cols-2">
        {config.availablePhpVersions.map((version) => (
          <div
            className="border border-voicebox-border bg-voicebox-surface p-3"
            key={version.version}
          >
            <div className="flex items-center justify-between gap-3">
              <span className="font-mono text-sm text-voicebox-black">{version.label}</span>
              <span className="font-mono text-xs uppercase text-voicebox-secondary">
                {supportPhaseLabels[version.supportPhase]}
              </span>
            </div>
            <p className="mt-2 font-mono text-xs uppercase text-voicebox-secondary">
              {version.installed ? "Installed" : "Not installed"}
            </p>
            {version.binaryDisplayName ? (
              <p className="mt-2 break-words font-mono text-xs text-voicebox-secondary">
                {version.binaryDisplayName}
              </p>
            ) : null}
            {version.lifecycleWarning ? (
              <p className="mt-2 border-l-2 border-voicebox-red pl-2 text-xs leading-relaxed text-voicebox-red">
                {version.lifecycleWarning}
              </p>
            ) : null}
          </div>
        ))}
      </div>

      {config.selectedPhpBinary ? (
        <p className="mt-5 border-l-2 border-voicebox-success pl-3 font-mono text-xs leading-relaxed text-voicebox-success">
          Active binary: {config.selectedPhpBinary.displayName}
        </p>
      ) : null}

      <p className="mt-5 border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
        {config.statusMessage}
      </p>
    </section>
  );
}
