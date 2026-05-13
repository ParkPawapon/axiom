import type { Project, ProjectDraft } from "../types/project.types";
import { ProjectCard } from "./project-card";

interface ProjectListProps {
  activeProjectId?: string;
  isBusy: boolean;
  projects: Project[];
  onDelete: (projectId: string) => Promise<void>;
  onSelect: (projectId: string) => void;
  onUpdate: (projectId: string, draft: ProjectDraft) => Promise<void>;
}

export function ProjectList({
  activeProjectId,
  isBusy,
  onDelete,
  onSelect,
  onUpdate,
  projects,
}: ProjectListProps) {
  return (
    <section className="grid gap-3">
      {projects.map((project) => (
        <ProjectCard
          isActive={project.id === activeProjectId}
          isBusy={isBusy}
          key={project.id}
          project={project}
          onDelete={onDelete}
          onSelect={onSelect}
          onUpdate={onUpdate}
        />
      ))}
    </section>
  );
}
