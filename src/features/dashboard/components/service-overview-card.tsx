import { Chip } from "../../../shared/components/ui/chip";

interface ServiceOverviewCardProps {
  failedCount: number;
  runningCount: number;
  totalCount: number;
}

function getTone(failedCount: number, runningCount: number) {
  if (failedCount > 0) {
    return "error";
  }

  if (runningCount > 0) {
    return "success";
  }

  return "neutral";
}

export function ServiceOverviewCard({
  failedCount,
  runningCount,
  totalCount,
}: ServiceOverviewCardProps) {
  return (
    <article className="border-2 border-voicebox-black bg-white p-5">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">Services</p>
          <p className="mt-2 font-display text-5xl uppercase leading-none text-voicebox-black">
            {runningCount}/{totalCount}
          </p>
        </div>
        <Chip tone={getTone(failedCount, runningCount)}>{failedCount} failed</Chip>
      </div>
      <p className="mt-5 text-sm text-voicebox-secondary">
        Service actions remain guarded by backend capability checks and allowlisted adapters.
      </p>
    </article>
  );
}
