import { useState } from "react";
import type { FormEvent } from "react";

import { Button } from "../../../shared/components/ui/button";
import { Input } from "../../../shared/components/ui/input";
import type { ProjectDraft } from "../types/project.types";
import { ProjectPathPicker } from "./project-path-picker";

interface ProjectFormShellProps {
  isBusy: boolean;
  onCreate: (draft: ProjectDraft) => Promise<void>;
}

const emptyDraft: ProjectDraft = {
  name: "",
  documentRoot: "",
};

export function ProjectFormShell({ isBusy, onCreate }: ProjectFormShellProps) {
  const [draft, setDraft] = useState<ProjectDraft>(emptyDraft);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    await onCreate(draft);
    setDraft(emptyDraft);
  }

  return (
    <form className="border-2 border-voicebox-black bg-white p-5" onSubmit={handleSubmit}>
      <div className="border-b border-voicebox-border pb-4">
        <p className="font-mono text-xs uppercase text-voicebox-secondary">Project Config</p>
        <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
          Add Project
        </h2>
      </div>

      <div className="mt-5 grid gap-4">
        <label className="grid gap-2">
          <span className="text-sm font-bold text-voicebox-black">Project name</span>
          <Input
            disabled={isBusy}
            placeholder="Local app"
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

        <div>
          <Button disabled={isBusy} type="submit">
            {isBusy ? "Saving" : "Add Project"}
          </Button>
        </div>
      </div>
    </form>
  );
}
