import { Chip } from "../../../shared/components/ui/chip";

interface ProjectSummaryCardProps {
  configuredCount: number;
  runningCount: number;
  totalCount: number;
}

export function ProjectSummaryCard({
  configuredCount,
  runningCount,
  totalCount,
}: ProjectSummaryCardProps) {
  return (
    <article className="border-2 border-voicebox-black bg-white p-5">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">Projects</p>
          <p className="mt-2 font-display text-5xl uppercase leading-none text-voicebox-black">
            {totalCount}
          </p>
        </div>
        <Chip tone={runningCount > 0 ? "success" : "neutral"}>{runningCount} running</Chip>
      </div>
      <p className="mt-5 text-sm text-voicebox-secondary">
        {configuredCount} project profiles have a selected PHP binary.
      </p>
    </article>
  );
}
