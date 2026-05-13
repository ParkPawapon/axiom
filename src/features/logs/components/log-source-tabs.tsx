import { Chip } from "../../../shared/components/ui/chip";
import { ToolbarButton } from "../../../shared/components/ui/toolbar-button";
import type { ProjectLogSource } from "../types/log.types";

interface LogSourceTabsProps {
  activeSourceId?: string;
  sources: ProjectLogSource[];
  onSelect: (sourceId: string) => void;
}

export function LogSourceTabs({ activeSourceId, onSelect, sources }: LogSourceTabsProps) {
  return (
    <div className="flex gap-2 overflow-x-auto border-b border-voicebox-border pb-3">
      {sources.map((source) => (
        <ToolbarButton
          className="gap-2"
          isActive={source.projectId === activeSourceId}
          key={source.projectId}
          onClick={() => onSelect(source.projectId)}
        >
          {source.projectName}
          <Chip tone={source.processState === "running" ? "success" : "neutral"}>
            {source.processState}
          </Chip>
        </ToolbarButton>
      ))}
    </div>
  );
}
