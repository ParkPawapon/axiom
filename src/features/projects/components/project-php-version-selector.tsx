import { Button } from "../../../shared/components/ui/button";
import { Select } from "../../../shared/components/ui/select";
import type { PhpVersionSupportPhase, ProjectPhpVersionConfig } from "../types/project.types";

interface ProjectPhpVersionSelectorProps {
  config: ProjectPhpVersionConfig;
  draftVersion: string;
  isSaving: boolean;
  onDraftVersionChange: (version: string) => void;
  onSave: () => void;
}

const supportPhaseLabels: Record<PhpVersionSupportPhase, string> = {
  active: "ACTIVE SUPPORT",
  security: "SECURITY SUPPORT",
};

export function ProjectPhpVersionSelector({
  config,
  draftVersion,
  isSaving,
  onDraftVersionChange,
  onSave,
}: ProjectPhpVersionSelectorProps) {
  const hasChanges = draftVersion !== config.selectedPhpVersion;

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
            disabled={isSaving}
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

        <Button
          className="self-end"
          disabled={!hasChanges || isSaving}
          onClick={onSave}
          variant="primary"
        >
          {isSaving ? "Saving" : "Apply"}
        </Button>
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
          </div>
        ))}
      </div>

      <p className="mt-5 border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
        {config.statusMessage}
      </p>
    </section>
  );
}
