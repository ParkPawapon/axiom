import { EmptyState } from "../../../shared/components/ui/empty-state";
import type { LogEntry } from "../../logs/types/log.types";
import { LogLine } from "./log-line";

interface UnifiedLogViewerProps {
  readonly entries: LogEntry[];
  readonly statusMessage: string;
  readonly truncated: boolean;
}

export function UnifiedLogViewer({
  entries,
  statusMessage,
  truncated,
}: UnifiedLogViewerProps) {
  if (entries.length === 0) {
    return (
      <EmptyState
        description={statusMessage}
        title="No log lines"
      />
    );
  }

  return (
    <section className="border-2 border-voicebox-black bg-white">
      <div className="flex items-center justify-between gap-3 border-b-2 border-voicebox-black p-3">
        <p className="text-sm font-semibold text-voicebox-black">{statusMessage}</p>
        {truncated ? (
          <span className="font-mono text-xs uppercase text-voicebox-warning">Truncated</span>
        ) : null}
      </div>
      <div className="max-h-[34rem] overflow-auto px-3">
        {entries.map((entry) => (
          <LogLine entry={entry} key={entry.id} />
        ))}
      </div>
    </section>
  );
}
