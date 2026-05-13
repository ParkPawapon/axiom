import { useEffect, useState } from "react";

import { Button } from "../../../shared/components/ui/button";
import { Input } from "../../../shared/components/ui/input";
import type { Project, ProjectDraft } from "../types/project.types";
import { ProjectPathPicker } from "./project-path-picker";

interface ProjectCardProps {
  isActive: boolean;
  isBusy: boolean;
  isSelectedForAction: boolean;
  project: Project;
  onDelete: (projectId: string) => Promise<void>;
  onSelect: (projectId: string) => void;
  onToggleActionSelection: (projectId: string) => void;
  onUpdate: (projectId: string, draft: ProjectDraft) => Promise<void>;
}

export function ProjectCard({
  isActive,
  isBusy,
  isSelectedForAction,
  onDelete,
  onSelect,
  onToggleActionSelection,
  onUpdate,
  project,
}: ProjectCardProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [draft, setDraft] = useState<ProjectDraft>({
    name: project.name,
    documentRoot: project.documentRoot,
  });

  useEffect(() => {
    setDraft({ name: project.name, documentRoot: project.documentRoot });
  }, [project.documentRoot, project.name]);

  async function handleSave() {
    await onUpdate(project.id, draft);
    setIsEditing(false);
  }

  return (
    <article
      className={`border bg-white p-4 ${
        isActive ? "border-2 border-voicebox-black" : "border-voicebox-border"
      }`}
    >
      {isEditing ? (
        <div className="grid gap-4">
          <label className="grid gap-2">
            <span className="text-sm font-bold text-voicebox-black">Project name</span>
            <Input
              disabled={isBusy}
              value={draft.name}
              onChange={(event) =>
                setDraft((currentDraft) => ({
                  ...currentDraft,
                  name: event.currentTarget.value,
                }))
              }
            />
          </label>
          <ProjectPathPicker
            disabled={isBusy}
            label="Document root"
            value={draft.documentRoot}
            onChange={(documentRoot) =>
              setDraft((currentDraft) => ({ ...currentDraft, documentRoot }))
            }
          />
          <div className="flex flex-wrap gap-2">
            <Button disabled={isBusy} onClick={() => void handleSave()}>
              Save
            </Button>
            <Button disabled={isBusy} onClick={() => setIsEditing(false)} variant="secondary">
              Cancel
            </Button>
          </div>
        </div>
      ) : (
        <div className="grid gap-4">
          <label className="flex items-center gap-3 border border-voicebox-border bg-voicebox-surface p-2 font-mono text-xs uppercase text-voicebox-secondary">
            <input
              checked={isSelectedForAction}
              disabled={isBusy}
              onChange={() => onToggleActionSelection(project.id)}
              type="checkbox"
            />
            Process action target
          </label>
          <button
            className="grid gap-2 text-left"
            onClick={() => onSelect(project.id)}
            type="button"
          >
            <span className="font-display text-xl uppercase leading-none text-voicebox-black">
              {project.name}
            </span>
            <span className="break-words font-mono text-xs text-voicebox-secondary">
              {project.documentRoot}
            </span>
          </button>
          <div className="flex flex-wrap gap-2">
            <Button disabled={isBusy} onClick={() => onSelect(project.id)} variant="secondary">
              Select
            </Button>
            <Button disabled={isBusy} onClick={() => setIsEditing(true)} variant="secondary">
              Edit
            </Button>
            <Button disabled={isBusy} onClick={() => void onDelete(project.id)} variant="ghost">
              Delete
            </Button>
          </div>
        </div>
      )}
    </article>
  );
}
