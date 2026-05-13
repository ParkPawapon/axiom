import { Chip } from "../../../shared/components/ui/chip";
import type { DetectedPhpBinary } from "../../projects/types/project.types";

interface RuntimePathCardProps {
  binary?: DetectedPhpBinary;
  projectName: string;
  selectedVersion: string;
}

export function RuntimePathCard({ binary, projectName, selectedVersion }: RuntimePathCardProps) {
  return (
    <article className="border border-voicebox-border bg-white p-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">Project Runtime</p>
          <h3 className="mt-1 font-display text-xl uppercase leading-none text-voicebox-black">
            {projectName}
          </h3>
        </div>
        <Chip tone={binary ? "success" : "warning"}>PHP {selectedVersion}</Chip>
      </div>
      <p className="mt-4 break-words font-mono text-xs text-voicebox-secondary">
        {binary
          ? `${binary.displayName} (${binary.path})`
          : "No selected binary. Install or switch PHP from the Projects workflow."}
      </p>
    </article>
  );
}
