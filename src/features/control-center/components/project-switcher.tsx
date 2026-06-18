import { Select } from "../../../shared/components/ui/select";
import type { ControlCenterProjectSummary } from "../types/control-center.types";

interface ProjectSwitcherProps {
  projects: ControlCenterProjectSummary[];
  selectedProject?: ControlCenterProjectSummary;
  onProjectChange: (projectId: string) => void;
}

export function ProjectSwitcher({
  projects,
  selectedProject,
  onProjectChange,
}: ProjectSwitcherProps) {
  return (
    <section className="border-2 border-voicebox-black bg-white p-4">
      <div className="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">Selected Project</p>
          <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
            {selectedProject?.name ?? "No Project"}
          </h2>
          <p className="mt-2 break-all text-xs text-voicebox-secondary">
            {selectedProject?.documentRoot ?? "Add a project to start local development."}
          </p>
        </div>
        <Select
          aria-label="Select project"
          disabled={projects.length === 0}
          onChange={(event) => onProjectChange(event.target.value)}
          value={selectedProject?.id ?? ""}
        >
          {projects.length === 0 ? <option value="">No projects</option> : null}
          {projects.map((project) => (
            <option key={project.id} value={project.id}>
              {project.name}
            </option>
          ))}
        </Select>
      </div>
    </section>
  );
}
