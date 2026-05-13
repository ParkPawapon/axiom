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
          isActive={source.projectId === activeSourceId}
          key={source.projectId}
          onClick={() => onSelect(source.projectId)}
        >
          {source.projectName}
        </ToolbarButton>
      ))}
    </div>
  );
}
