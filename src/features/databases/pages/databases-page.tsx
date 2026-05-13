import { useCallback, useEffect, useMemo, useState } from "react";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { Button } from "../../../shared/components/ui/button";
import { EmptyState } from "../../../shared/components/ui/empty-state";
import { Input } from "../../../shared/components/ui/input";
import { Select } from "../../../shared/components/ui/select";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import { listProjects } from "../../projects/api/project.commands";
import type { Project } from "../../projects/types/project.types";
import { listServices } from "../../services/api/service.commands";
import type { ManagedService } from "../../services/types/service.types";
import {
  backupProjectDatabase,
  createProjectDatabaseMigration,
  listProjectDatabaseProfiles,
  provisionProjectDatabase,
  restoreProjectDatabase,
  runProjectDatabaseMigrations,
} from "../api/database.commands";
import { DatabaseServiceCard } from "../components/database-service-card";
import { MysqlStatusPanel } from "../components/mysql-status-panel";
import { PostgresStatusPanel } from "../components/postgres-status-panel";
import type { DatabaseType, ProjectDatabaseProfile } from "../types/database.types";

const databaseTypes = ["mysql", "postgresql"] as const satisfies readonly DatabaseType[];

type ActionName = "backup" | "migration:create" | "migration:run" | "provision" | "restore";
type ActionKey = `${ActionName}:${DatabaseType}`;

export function DatabasesPage() {
  const [actionKey, setActionKey] = useState<ActionKey>();
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isLoading, setIsLoading] = useState(true);
  const [migrationNames, setMigrationNames] = useState<Record<DatabaseType, string>>({
    mysql: "",
    postgresql: "",
  });
  const [noticeMessage, setNoticeMessage] = useState<string>();
  const [profiles, setProfiles] = useState<ProjectDatabaseProfile[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [restorePaths, setRestorePaths] = useState<Record<DatabaseType, string>>({
    mysql: "",
    postgresql: "",
  });
  const [selectedProjectId, setSelectedProjectId] = useState("");
  const [services, setServices] = useState<ManagedService[]>([]);

  const loadInventory = useCallback(async () => {
    setIsLoading(true);
    setErrorMessage(undefined);

    try {
      const [loadedProjects, loadedServices] = await Promise.all([listProjects(), listServices()]);
      setProjects(loadedProjects);
      setServices(loadedServices);
      setSelectedProjectId((currentProjectId) => currentProjectId || loadedProjects[0]?.id || "");
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "Database inventory could not be loaded safely."));
    } finally {
      setIsLoading(false);
    }
  }, []);

  const loadProfiles = useCallback(async (projectId: string) => {
    if (!projectId) {
      setProfiles([]);
      return;
    }

    setErrorMessage(undefined);

    try {
      setProfiles(await listProjectDatabaseProfiles(projectId));
    } catch (error) {
      setErrorMessage(getErrorMessage(error, "Database profiles could not be loaded safely."));
    }
  }, []);

  useEffect(() => {
    void loadInventory();
  }, [loadInventory]);

  useEffect(() => {
    void loadProfiles(selectedProjectId);
  }, [loadProfiles, selectedProjectId]);

  const databaseServices = useMemo(
    () =>
      services.filter(
        (service) => service.serviceType === "mysql" || service.serviceType === "postgresql",
      ),
    [services],
  );

  const selectedProject = projects.find((project) => project.id === selectedProjectId);

  const profileByType = useMemo(
    () =>
      profiles.reduce<Partial<Record<DatabaseType, ProjectDatabaseProfile>>>(
        (profilesByType, profile) => {
          profilesByType[profile.databaseType] = profile;
          return profilesByType;
        },
        {},
      ),
    [profiles],
  );

  const runAction = useCallback(
    async (nextActionKey: ActionKey, action: () => Promise<string>) => {
      setActionKey(nextActionKey);
      setErrorMessage(undefined);
      setNoticeMessage(undefined);

      try {
        const message = await action();
        setNoticeMessage(message);
        await loadProfiles(selectedProjectId);
      } catch (error) {
        setErrorMessage(getErrorMessage(error, "Database action failed safely."));
      } finally {
        setActionKey(undefined);
      }
    },
    [loadProfiles, selectedProjectId],
  );

  const provisionDatabase = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(`provision:${databaseType}`, async () => {
      const result = await provisionProjectDatabase(selectedProjectId, databaseType);
      return result.statusMessage;
    });
  };

  const backupDatabase = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(`backup:${databaseType}`, async () => {
      const result = await backupProjectDatabase(selectedProjectId, databaseType);
      return `${result.statusMessage} ${result.backupPath}`;
    });
  };

  const restoreDatabase = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    const backupPath = restorePaths[databaseType].trim();

    void runAction(`restore:${databaseType}`, async () => {
      const result = await restoreProjectDatabase(selectedProjectId, databaseType, backupPath);
      return `${result.statusMessage} ${result.backupPath}`;
    });
  };

  const createMigration = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    const migrationName = migrationNames[databaseType].trim();

    void runAction(`migration:create:${databaseType}`, async () => {
      const result = await createProjectDatabaseMigration(
        selectedProjectId,
        databaseType,
        migrationName,
      );
      return `${result.statusMessage} ${result.migrationPath}`;
    });
  };

  const runMigrations = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(`migration:run:${databaseType}`, async () => {
      const result = await runProjectDatabaseMigrations(selectedProjectId, databaseType);
      return result.statusMessage;
    });
  };

  return (
    <PageShell
      title="Databases"
      description="Project database provisioning creates app-owned data directories, stores credentials in the OS secure store, and executes database CLIs only through backend allowlists."
    >
      {errorMessage ? <ErrorPanel message={errorMessage} /> : null}
      {noticeMessage ? (
        <div className="border-2 border-voicebox-black bg-white p-4 text-sm font-semibold text-voicebox-black">
          {noticeMessage}
        </div>
      ) : null}
      {isLoading ? <LoadingState label="Loading database inventory" /> : null}

      <section className="grid gap-4 border-2 border-voicebox-black bg-white p-5">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">Project scope</p>
          <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
            Database Provisioning
          </h2>
        </div>

        {projects.length > 0 ? (
          <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
            Project
            <Select
              value={selectedProjectId}
              onChange={(event) => setSelectedProjectId(event.target.value)}
            >
              {projects.map((project) => (
                <option key={project.id} value={project.id}>
                  {project.name}
                </option>
              ))}
            </Select>
          </label>
        ) : (
          <EmptyState
            title="No projects registered"
            description="Create a project before provisioning MySQL or PostgreSQL resources."
          />
        )}

        {selectedProject ? (
          <p className="border-l-2 border-voicebox-black pl-3 font-mono text-xs text-voicebox-secondary">
            {selectedProject.documentRoot}
          </p>
        ) : null}
      </section>

      <section className="grid gap-4 xl:grid-cols-2">
        {databaseTypes.map((databaseType) => {
          const profile = profileByType[databaseType];
          const ready = profile?.status === "ready";

          return (
            <article
              key={databaseType}
              className="grid gap-4 border-2 border-voicebox-black bg-white p-5"
            >
              <div className="flex items-start justify-between gap-3 border-b border-voicebox-border pb-4">
                <div>
                  <p className="font-mono text-xs uppercase text-voicebox-secondary">
                    {databaseType}
                  </p>
                  <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
                    {databaseType === "mysql" ? "MySQL" : "PostgreSQL"}
                  </h2>
                </div>
                <span className={statusClassName(profile?.status)}>
                  {statusLabel(profile?.status)}
                </span>
              </div>

              {profile ? (
                <dl className="grid gap-3 text-sm">
                  <div>
                    <dt className="font-mono text-xs uppercase text-voicebox-tertiary">Database</dt>
                    <dd className="font-mono text-xs text-voicebox-black">
                      {profile.databaseName}
                    </dd>
                  </div>
                  <div>
                    <dt className="font-mono text-xs uppercase text-voicebox-tertiary">User</dt>
                    <dd className="font-mono text-xs text-voicebox-black">{profile.username}</dd>
                  </div>
                  <div>
                    <dt className="font-mono text-xs uppercase text-voicebox-tertiary">Endpoint</dt>
                    <dd className="font-mono text-xs text-voicebox-black">
                      {profile.host}:{profile.port}
                    </dd>
                  </div>
                  {profile.adminUrl ? (
                    <div>
                      <dt className="font-mono text-xs uppercase text-voicebox-tertiary">
                        Admin URL
                      </dt>
                      <dd>
                        <a
                          className="font-mono text-xs font-bold text-voicebox-black underline"
                          href={profile.adminUrl}
                          rel="noreferrer"
                          target="_blank"
                        >
                          {profile.adminUrl}
                        </a>
                      </dd>
                    </div>
                  ) : null}
                  <div>
                    <dt className="font-mono text-xs uppercase text-voicebox-tertiary">Data dir</dt>
                    <dd className="break-all font-mono text-xs text-voicebox-secondary">
                      {profile.dataDir}
                    </dd>
                  </div>
                </dl>
              ) : (
                <p className="text-sm text-voicebox-secondary">
                  No database profile exists for this project yet.
                </p>
              )}

              {profile?.statusMessage ? (
                <p className="border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
                  {profile.statusMessage}
                </p>
              ) : null}

              <div className="flex flex-wrap gap-2">
                <Button
                  disabled={!selectedProjectId || actionKey === `provision:${databaseType}`}
                  onClick={() => provisionDatabase(databaseType)}
                >
                  Provision
                </Button>
                <Button
                  disabled={!ready || actionKey === `backup:${databaseType}`}
                  onClick={() => backupDatabase(databaseType)}
                  variant="secondary"
                >
                  Backup
                </Button>
                <Button
                  disabled={!ready || actionKey === `migration:run:${databaseType}`}
                  onClick={() => runMigrations(databaseType)}
                  variant="secondary"
                >
                  Run migrations
                </Button>
              </div>

              <div className="grid gap-2">
                <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
                  Migration name
                  <Input
                    value={migrationNames[databaseType]}
                    onChange={(event) =>
                      setMigrationNames((current) => ({
                        ...current,
                        [databaseType]: event.target.value,
                      }))
                    }
                    placeholder="create users table"
                  />
                </label>
                <Button
                  disabled={
                    !profile ||
                    !migrationNames[databaseType].trim() ||
                    actionKey === `migration:create:${databaseType}`
                  }
                  onClick={() => createMigration(databaseType)}
                  variant="secondary"
                >
                  Create migration
                </Button>
              </div>

              <div className="grid gap-2">
                <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
                  Restore .sql path
                  <Input
                    value={restorePaths[databaseType]}
                    onChange={(event) =>
                      setRestorePaths((current) => ({
                        ...current,
                        [databaseType]: event.target.value,
                      }))
                    }
                    placeholder="/absolute/path/backup.sql"
                  />
                </label>
                <Button
                  disabled={
                    !ready ||
                    !restorePaths[databaseType].trim() ||
                    actionKey === `restore:${databaseType}`
                  }
                  onClick={() => restoreDatabase(databaseType)}
                  variant="secondary"
                >
                  Restore
                </Button>
              </div>
            </article>
          );
        })}
      </section>

      <div className="grid gap-5 xl:grid-cols-2">
        <MysqlStatusPanel />
        <PostgresStatusPanel />
      </div>

      {databaseServices.length > 0 ? (
        <section className="grid gap-4 xl:grid-cols-2">
          {databaseServices.map((service) => (
            <DatabaseServiceCard key={service.id} service={service} />
          ))}
        </section>
      ) : null}
    </PageShell>
  );
}

function statusLabel(status: ProjectDatabaseProfile["status"] | undefined) {
  if (status === "ready") {
    return "Ready";
  }

  if (status === "pending") {
    return "Pending";
  }

  if (status === "failed") {
    return "Failed";
  }

  return "Not provisioned";
}

function statusClassName(status: ProjectDatabaseProfile["status"] | undefined) {
  const baseClassName = "border px-2 py-1 font-mono text-xs uppercase";

  if (status === "ready") {
    return `${baseClassName} border-voicebox-success text-voicebox-success`;
  }

  if (status === "pending") {
    return `${baseClassName} border-voicebox-warning text-voicebox-warning`;
  }

  if (status === "failed") {
    return `${baseClassName} border-voicebox-red text-voicebox-red`;
  }

  return `${baseClassName} border-voicebox-border text-voicebox-secondary`;
}
