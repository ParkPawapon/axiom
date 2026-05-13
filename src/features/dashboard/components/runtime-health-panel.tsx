import { Chip } from "../../../shared/components/ui/chip";

interface RuntimeHealthPanelProps {
  missingBinaryCount: number;
  selectedVersionCount: number;
  totalProjectCount: number;
}

export function RuntimeHealthPanel({
  missingBinaryCount,
  selectedVersionCount,
  totalProjectCount,
}: RuntimeHealthPanelProps) {
  return (
    <article className="border-2 border-voicebox-black bg-white p-5">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">Runtime Health</p>
          <p className="mt-2 font-display text-5xl uppercase leading-none text-voicebox-black">
            {selectedVersionCount}/{totalProjectCount}
          </p>
        </div>
        <Chip tone={missingBinaryCount > 0 ? "warning" : "success"}>
          {missingBinaryCount} missing
        </Chip>
      </div>
      <p className="mt-5 text-sm text-voicebox-secondary">
        PHP version selection is persisted per project. Missing binaries require explicit install
        confirmation.
      </p>
    </article>
  );
}
