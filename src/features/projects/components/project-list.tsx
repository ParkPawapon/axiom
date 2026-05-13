import type { Project, ProjectDraft } from "../types/project.types";
import { ProjectCard } from "./project-card";

interface ProjectListProps {
  activeProjectId?: string;
  isBusy: boolean;
  projects: Project[];
  selectedActionProjectIds: string[];
  onDelete: (projectId: string) => Promise<void>;
  onSelect: (projectId: string) => void;
  onToggleActionSelection: (projectId: string) => void;
  onUpdate: (projectId: string, draft: ProjectDraft) => Promise<void>;
}

export function ProjectList({
  activeProjectId,
  isBusy,
  onDelete,
  onSelect,
  onToggleActionSelection,
  onUpdate,
  projects,
  selectedActionProjectIds,
}: ProjectListProps) {
  return (
    <section className="grid gap-3">
      {projects.map((project) => (
        <ProjectCard
          isActive={project.id === activeProjectId}
          isBusy={isBusy}
          isSelectedForAction={selectedActionProjectIds.includes(project.id)}
          key={project.id}
          project={project}
          onDelete={onDelete}
          onSelect={onSelect}
          onToggleActionSelection={onToggleActionSelection}
          onUpdate={onUpdate}
        />
      ))}
    </section>
  );
}
