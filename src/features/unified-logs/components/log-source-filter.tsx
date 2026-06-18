import { Button } from "../../../shared/components/ui/button";
import type { UnifiedLogSource, UnifiedLogSourceId } from "../types/unified-log.types";

interface LogSourceFilterProps {
  readonly activeSourceId: UnifiedLogSourceId;
  readonly onSourceChange: (sourceId: UnifiedLogSourceId) => void;
  readonly sources: UnifiedLogSource[];
}

export function LogSourceFilter({
  activeSourceId,
  onSourceChange,
  sources,
}: LogSourceFilterProps) {
  return (
    <div aria-label="Log sources" className="flex flex-wrap gap-2" role="group">
      {sources.map((source) => {
        const isActive = source.id === activeSourceId;

        return (
          <Button
            aria-pressed={isActive}
            className="h-10"
            key={source.id}
            onClick={() => onSourceChange(source.id)}
            variant={isActive ? "primary" : "secondary"}
          >
            {source.label}
          </Button>
        );
      })}
    </div>
  );
}
