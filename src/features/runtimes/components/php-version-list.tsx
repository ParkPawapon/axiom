import { Chip } from "../../../shared/components/ui/chip";
import type { PhpVersionOption } from "../../projects/types/project.types";

interface PhpVersionListProps {
  versions: PhpVersionOption[];
}

const phaseTone: Record<PhpVersionOption["supportPhase"], "success" | "warning" | "error"> = {
  active: "success",
  endOfLife: "error",
  security: "warning",
};

const phaseLabel: Record<PhpVersionOption["supportPhase"], string> = {
  active: "Active",
  endOfLife: "End of life",
  security: "Security",
};

export function PhpVersionList({ versions }: PhpVersionListProps) {
  return (
    <section className="border-2 border-voicebox-black bg-white p-5">
      <div className="border-b border-voicebox-border pb-4">
        <p className="font-mono text-xs uppercase text-voicebox-secondary">PHP Catalog</p>
        <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
          Available Branches
        </h2>
      </div>
      <div className="mt-5 grid gap-3 md:grid-cols-2">
        {versions.map((version) => (
          <article
            className="border border-voicebox-border bg-voicebox-surface p-3"
            key={version.version}
          >
            <div className="flex items-center justify-between gap-3">
              <p className="font-mono text-sm text-voicebox-black">{version.label}</p>
              <Chip tone={phaseTone[version.supportPhase]}>{phaseLabel[version.supportPhase]}</Chip>
            </div>
            <p className="mt-3 font-mono text-xs uppercase text-voicebox-secondary">
              {version.installed ? "Installed" : "Not installed"}
              {version.recommended ? " / Recommended" : ""}
            </p>
            {version.binaryDisplayName ? (
              <p className="mt-2 break-words font-mono text-xs text-voicebox-secondary">
                {version.binaryDisplayName}
              </p>
            ) : null}
          </article>
        ))}
      </div>
    </section>
  );
}
