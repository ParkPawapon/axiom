import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import { ErrorPanel } from "../../../shared/components/feedback/error-panel";
import { LoadingState } from "../../../shared/components/feedback/loading-state";
import { PageShell } from "../../../shared/components/layout/page-shell";
import { Button } from "../../../shared/components/ui/button";
import { EmptyState } from "../../../shared/components/ui/empty-state";
import { Input } from "../../../shared/components/ui/input";
import { Select } from "../../../shared/components/ui/select";
import { formatDate } from "../../../shared/utils/format-date";
import { getErrorMessage } from "../../../shared/utils/get-error-message";
import { listProjects } from "../../projects/api/project.commands";
import type { Project } from "../../projects/types/project.types";
import { listServices } from "../../services/api/service.commands";
import type { ManagedService } from "../../services/types/service.types";
import {
  backupProjectDatabase,
  createProjectDatabaseMigration,
  exportDatabaseBackupTrustBundle,
  generateProjectDatabaseMigrationRollback,
  getDatabaseBackupKeyManagementStatus,
  getDatabaseBackupSchedulerStatus,
  importDatabaseBackupTrustBundle,
  installDatabaseBackupScheduler,
  listDatabaseBackupDestinations,
  listDatabaseBackupPolicies,
  listProjectDatabaseProfiles,
  provisionProjectDatabase,
  restoreProjectDatabaseToPointInTime,
  restoreProjectDatabase,
  restoreProjectDatabaseWithReplay,
  rollbackProjectDatabaseMigrations,
  runDueDatabaseBackups,
  runProjectDatabaseMigrations,
  uninstallDatabaseBackupScheduler,
  updateDatabaseBackupPolicy,
  updateDatabaseBackupDestination,
} from "../api/database.commands";
import { DatabaseBackupControls } from "../components/database-backup-controls";
import { DatabaseServiceCard } from "../components/database-service-card";
import { MysqlStatusPanel } from "../components/mysql-status-panel";
import { PostgresStatusPanel } from "../components/postgres-status-panel";
import type {
  DatabaseProvisioningResult,
  DatabaseBackupOptions,
  DatabaseBackupKeyManagementStatus,
  DatabaseBackupPolicy,
  DatabaseBackupPolicyUpdate,
  DatabaseBackupRemoteDestination,
  DatabaseBackupRemoteDestinationUpdate,
  DatabaseBackupSchedulerStatus,
  DatabaseType,
  DatabaseMigrationRollbackGenerationResult,
  ManagedDatabaseDependencyStatus,
  ManagedDatabasePackage,
  ProjectDatabaseProfile,
} from "../types/database.types";

const databaseTypes = ["mysql", "postgresql"] as const satisfies readonly DatabaseType[];

type ActionName =
  | "backup"
  | "migration:create"
  | "migration:generateRollback"
  | "migration:rollback"
  | "migration:run"
  | "provision"
  | "remote:save"
  | "restore:pitr"
  | "restore:replay"
  | "restore"
  | "schedule:save";
type ActionKey =
  | `${ActionName}:${DatabaseType}`
  | "schedule:runDue"
  | "scheduler:install"
  | "scheduler:uninstall"
  | "trust:export"
  | "trust:import"
  | "trust:refresh";

const defaultBackupOptions: DatabaseBackupOptions = {
  compression: "gzip",
  encryption: "aes256Gcm",
  retentionDays: 30,
};

const defaultPolicyInput: DatabaseBackupPolicyUpdate = {
  ...defaultBackupOptions,
  enabled: false,
  intervalMinutes: 1440,
};

const defaultRemoteDestinationInput: DatabaseBackupRemoteDestinationUpdate = {
  destinationPath: "",
  enabled: false,
  provider: "localPath",
};

export function DatabasesPage() {
  const [actionKey, setActionKey] = useState<ActionKey>();
  const isBackupSweepRunning = useRef(false);
  const [errorMessage, setErrorMessage] = useState<string>();
  const [isLoading, setIsLoading] = useState(true);
  const [keyManagementStatus, setKeyManagementStatus] =
    useState<DatabaseBackupKeyManagementStatus>();
  const [backupOptions, setBackupOptions] = useState<Record<DatabaseType, DatabaseBackupOptions>>({
    mysql: defaultBackupOptions,
    postgresql: defaultBackupOptions,
  });
  const [backupPolicies, setBackupPolicies] = useState<
    Partial<Record<DatabaseType, DatabaseBackupPolicy>>
  >({});
  const [backupDestinations, setBackupDestinations] = useState<
    Partial<Record<DatabaseType, DatabaseBackupRemoteDestination>>
  >({});
  const [migrationNames, setMigrationNames] = useState<Record<DatabaseType, string>>({
    mysql: "",
    postgresql: "",
  });
  const [noticeMessage, setNoticeMessage] = useState<string>();
  const [pointInTimeTargets, setPointInTimeTargets] = useState<Record<DatabaseType, string>>({
    mysql: "",
    postgresql: "",
  });
  const [profiles, setProfiles] = useState<ProjectDatabaseProfile[]>([]);
  const [provisioningResults, setProvisioningResults] = useState<
    Partial<Record<DatabaseType, DatabaseProvisioningResult>>
  >({});
  const [projects, setProjects] = useState<Project[]>([]);
  const [restorePaths, setRestorePaths] = useState<Record<DatabaseType, string>>({
    mysql: "",
    postgresql: "",
  });
  const [replaySourcePaths, setReplaySourcePaths] = useState<Record<DatabaseType, string>>({
    mysql: "",
    postgresql: "",
  });
  const [rollbackGenerationPaths, setRollbackGenerationPaths] = useState<
    Record<DatabaseType, string>
  >({
    mysql: "",
    postgresql: "",
  });
  const [rollbackGenerationResults, setRollbackGenerationResults] = useState<
    Partial<Record<DatabaseType, DatabaseMigrationRollbackGenerationResult>>
  >({});
  const [rollbackSteps, setRollbackSteps] = useState<Record<DatabaseType, number>>({
    mysql: 1,
    postgresql: 1,
  });
  const [schedulerStatus, setSchedulerStatus] = useState<DatabaseBackupSchedulerStatus>();
  const [remoteDestinationInputs, setRemoteDestinationInputs] = useState<
    Record<DatabaseType, DatabaseBackupRemoteDestinationUpdate>
  >({
    mysql: defaultRemoteDestinationInput,
    postgresql: defaultRemoteDestinationInput,
  });
  const [scheduleInputs, setScheduleInputs] = useState<
    Record<DatabaseType, DatabaseBackupPolicyUpdate>
  >({
    mysql: defaultPolicyInput,
    postgresql: defaultPolicyInput,
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

  const loadBackupPolicies = useCallback(async (projectId: string) => {
    if (!projectId) {
      setBackupPolicies({});
      return;
    }

    try {
      const policies = await listDatabaseBackupPolicies(projectId);
      const policiesByType = policies.reduce<Partial<Record<DatabaseType, DatabaseBackupPolicy>>>(
        (currentPolicies, policy) => {
          currentPolicies[policy.databaseType] = policy;
          return currentPolicies;
        },
        {},
      );

      setBackupPolicies(policiesByType);
      setScheduleInputs((currentInputs) => ({
        mysql: policyInputFromPolicy(policiesByType.mysql, currentInputs.mysql),
        postgresql: policyInputFromPolicy(policiesByType.postgresql, currentInputs.postgresql),
      }));
      setBackupOptions((currentOptions) => ({
        mysql: backupOptionsFromPolicy(policiesByType.mysql, currentOptions.mysql),
        postgresql: backupOptionsFromPolicy(policiesByType.postgresql, currentOptions.postgresql),
      }));
    } catch (error) {
      setErrorMessage(
        getErrorMessage(error, "Database backup policies could not be loaded safely."),
      );
    }
  }, []);

  const loadBackupDestinations = useCallback(async (projectId: string) => {
    if (!projectId) {
      setBackupDestinations({});
      return;
    }

    try {
      const destinations = await listDatabaseBackupDestinations(projectId);
      const destinationsByType = destinations.reduce<
        Partial<Record<DatabaseType, DatabaseBackupRemoteDestination>>
      >((currentDestinations, destination) => {
        currentDestinations[destination.databaseType] = destination;
        return currentDestinations;
      }, {});

      setBackupDestinations(destinationsByType);
      setRemoteDestinationInputs((currentInputs) => ({
        mysql: remoteDestinationInputFromDestination(destinationsByType.mysql, currentInputs.mysql),
        postgresql: remoteDestinationInputFromDestination(
          destinationsByType.postgresql,
          currentInputs.postgresql,
        ),
      }));
    } catch (error) {
      setErrorMessage(
        getErrorMessage(error, "Database backup destinations could not be loaded safely."),
      );
    }
  }, []);

  const loadSchedulerStatus = useCallback(async () => {
    try {
      setSchedulerStatus(await getDatabaseBackupSchedulerStatus());
    } catch (error) {
      setErrorMessage(
        getErrorMessage(error, "Database backup scheduler status could not be loaded safely."),
      );
    }
  }, []);

  const loadKeyManagementStatus = useCallback(async () => {
    try {
      setKeyManagementStatus(await getDatabaseBackupKeyManagementStatus());
    } catch (error) {
      setErrorMessage(
        getErrorMessage(error, "Database backup key status could not be loaded safely."),
      );
    }
  }, []);

  useEffect(() => {
    void loadInventory();
    void loadSchedulerStatus();
    void loadKeyManagementStatus();
  }, [loadInventory, loadKeyManagementStatus, loadSchedulerStatus]);

  useEffect(() => {
    void loadProfiles(selectedProjectId);
    void loadBackupPolicies(selectedProjectId);
    void loadBackupDestinations(selectedProjectId);
  }, [loadBackupDestinations, loadBackupPolicies, loadProfiles, selectedProjectId]);

  useEffect(() => {
    const intervalId = window.setInterval(() => {
      if (actionKey || isBackupSweepRunning.current) {
        return;
      }

      isBackupSweepRunning.current = true;

      void (async () => {
        try {
          const result = await runDueDatabaseBackups();

          if (result.completedBackups > 0) {
            setNoticeMessage(scheduledBackupNotice(result));
            await Promise.all([
              loadProfiles(selectedProjectId),
              loadBackupPolicies(selectedProjectId),
              loadBackupDestinations(selectedProjectId),
            ]);
          }

          if (result.errors.length > 0) {
            setErrorMessage(result.errors.join(" "));
          }
        } catch (error) {
          setErrorMessage(getErrorMessage(error, "Scheduled backup check failed safely."));
        } finally {
          isBackupSweepRunning.current = false;
        }
      })();
    }, 60_000);

    return () => window.clearInterval(intervalId);
  }, [actionKey, loadBackupDestinations, loadBackupPolicies, loadProfiles, selectedProjectId]);

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
        await loadBackupPolicies(selectedProjectId);
        await loadBackupDestinations(selectedProjectId);
        await loadSchedulerStatus();
      } catch (error) {
        setErrorMessage(getErrorMessage(error, "Database action failed safely."));
      } finally {
        setActionKey(undefined);
      }
    },
    [
      loadBackupDestinations,
      loadBackupPolicies,
      loadProfiles,
      loadSchedulerStatus,
      selectedProjectId,
    ],
  );

  const provisionDatabase = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(`provision:${databaseType}`, async () => {
      const result = await provisionProjectDatabase(selectedProjectId, databaseType);
      setProvisioningResults((currentResults) => ({
        ...currentResults,
        [databaseType]: result,
      }));
      return provisioningNotice(result);
    });
  };

  const backupDatabase = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(`backup:${databaseType}`, async () => {
      const result = await backupProjectDatabase(
        selectedProjectId,
        databaseType,
        backupOptions[databaseType],
      );
      return [
        result.statusMessage,
        result.backupPath,
        result.encrypted ? "Encrypted" : "Not encrypted",
        result.compressed ? "Compressed" : "Not compressed",
        result.signaturePath ? "Signed" : "Unsigned",
        result.remoteCopyPaths.length > 0
          ? `Copied to ${result.remoteCopyPaths.length} remote artifact(s).`
          : undefined,
        result.prunedBackupPaths.length > 0
          ? `Pruned ${result.prunedBackupPaths.length} expired artifact(s).`
          : undefined,
      ]
        .filter(Boolean)
        .join(" ");
    });
  };

  const pickRestorePath = (databaseType: DatabaseType) => {
    void (async () => {
      setErrorMessage(undefined);

      try {
        const selectedPath = await open({
          directory: false,
          filters: [
            {
              extensions: ["sql", "gz", "enc"],
              name: "Database backup",
            },
          ],
          multiple: false,
        });

        if (typeof selectedPath === "string") {
          setRestorePaths((currentPaths) => ({
            ...currentPaths,
            [databaseType]: selectedPath,
          }));
        }
      } catch (error) {
        setErrorMessage(getErrorMessage(error, "Restore file picker could not be opened safely."));
      }
    })();
  };

  const pickReplaySourcePath = (databaseType: DatabaseType) => {
    void (async () => {
      setErrorMessage(undefined);

      try {
        const selectedPath = await open({
          directory: true,
          multiple: false,
        });

        if (typeof selectedPath === "string") {
          setReplaySourcePaths((currentPaths) => ({
            ...currentPaths,
            [databaseType]: selectedPath,
          }));
        }
      } catch (error) {
        setErrorMessage(
          getErrorMessage(error, "Replay directory picker could not be opened safely."),
        );
      }
    })();
  };

  const restoreDatabase = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    const backupPath = restorePaths[databaseType].trim();

    void runAction(`restore:${databaseType}`, async () => {
      const result = await restoreProjectDatabase(selectedProjectId, databaseType, backupPath);
      return [
        result.statusMessage,
        `Source: ${result.restoredFromPath}`,
        result.decrypted ? "Decrypted" : undefined,
        result.decompressed ? "Decompressed" : undefined,
        result.signatureVerified ? "Signature verified" : "No signature verified",
      ]
        .filter(Boolean)
        .join(" ");
    });
  };

  const restorePointInTime = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    const targetTime = pointInTimeTargetToIso(pointInTimeTargets[databaseType]);
    if (!targetTime) {
      setErrorMessage("Point-in-time restore target is required.");
      return;
    }

    void runAction(`restore:pitr:${databaseType}`, async () => {
      const result = await restoreProjectDatabaseToPointInTime(
        selectedProjectId,
        databaseType,
        targetTime,
      );

      return [
        result.statusMessage,
        `Selected backup: ${result.selectedBackupPath}`,
        `Backup created: ${formatDate(result.selectedBackupCreatedAt)}`,
        result.restore.signatureVerified ? "Signature verified" : "No signature verified",
      ]
        .filter(Boolean)
        .join(" ");
    });
  };

  const restoreWithReplay = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    const baseBackupPath = restorePaths[databaseType].trim();
    const replaySourcePath = replaySourcePaths[databaseType].trim();
    const targetTime = pointInTimeTargetToIso(pointInTimeTargets[databaseType]);

    void runAction(`restore:replay:${databaseType}`, async () => {
      const result = await restoreProjectDatabaseWithReplay(
        selectedProjectId,
        databaseType,
        baseBackupPath,
        replaySourcePath,
        targetTime,
      );

      return [
        result.statusMessage,
        `Base: ${result.restore.restoredFromPath}`,
        result.replayedLogPaths.length > 0
          ? `Replayed ${result.replayedLogPaths.length} segment(s).`
          : "No replay segments matched the target.",
      ].join(" ");
    });
  };

  const saveBackupPolicy = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(`schedule:save:${databaseType}`, async () => {
      const update = scheduleInputs[databaseType];
      const result = await updateDatabaseBackupPolicy(selectedProjectId, databaseType, {
        ...update,
        compression: backupOptions[databaseType].compression,
        encryption: backupOptions[databaseType].encryption,
        retentionDays: update.retentionDays,
      });

      return result.statusMessage;
    });
  };

  const saveBackupDestination = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(`remote:save:${databaseType}`, async () => {
      const result = await updateDatabaseBackupDestination(
        selectedProjectId,
        databaseType,
        remoteDestinationInputs[databaseType],
      );

      return result.statusMessage;
    });
  };

  const runScheduledBackups = () => {
    void runAction("schedule:runDue", async () => {
      const result = await runDueDatabaseBackups();
      return scheduledBackupNotice(result);
    });
  };

  const installScheduler = () => {
    void runAction("scheduler:install", async () => {
      const result = await installDatabaseBackupScheduler();
      setSchedulerStatus(result.status);
      return result.statusMessage;
    });
  };

  const uninstallScheduler = () => {
    void runAction("scheduler:uninstall", async () => {
      const result = await uninstallDatabaseBackupScheduler();
      setSchedulerStatus(result.status);
      return result.statusMessage;
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

  const pickMigrationForRollbackGeneration = (databaseType: DatabaseType) => {
    void (async () => {
      setErrorMessage(undefined);

      try {
        const selectedPath = await open({
          directory: false,
          filters: [{ extensions: ["sql"], name: "SQL migration" }],
          multiple: false,
        });

        if (typeof selectedPath === "string") {
          setRollbackGenerationPaths((currentPaths) => ({
            ...currentPaths,
            [databaseType]: selectedPath,
          }));
        }
      } catch (error) {
        setErrorMessage(
          getErrorMessage(error, "Migration file picker could not be opened safely."),
        );
      }
    })();
  };

  const generateRollbackSql = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    const migrationPath = rollbackGenerationPaths[databaseType].trim();

    void runAction(`migration:generateRollback:${databaseType}`, async () => {
      const result = await generateProjectDatabaseMigrationRollback(
        selectedProjectId,
        databaseType,
        migrationPath,
      );
      setRollbackGenerationResults((currentResults) => ({
        ...currentResults,
        [databaseType]: result,
      }));

      return [
        result.statusMessage,
        result.rollbackPath,
        result.warnings.length > 0 ? `${result.warnings.length} warning(s).` : undefined,
      ]
        .filter(Boolean)
        .join(" ");
    });
  };

  const exportTrustBundle = () => {
    void (async () => {
      setErrorMessage(undefined);
      const selectedPath = await open({ directory: true, multiple: false });

      if (typeof selectedPath !== "string") {
        return;
      }

      void runAction("trust:export", async () => {
        const result = await exportDatabaseBackupTrustBundle(selectedPath);
        await loadKeyManagementStatus();
        return `${result.statusMessage} ${result.trustBundlePath}`;
      });
    })();
  };

  const importTrustBundle = () => {
    void (async () => {
      setErrorMessage(undefined);
      const selectedPath = await open({
        directory: false,
        filters: [{ extensions: ["json"], name: "Backup trust bundle" }],
        multiple: false,
      });

      if (typeof selectedPath !== "string") {
        return;
      }

      void runAction("trust:import", async () => {
        const result = await importDatabaseBackupTrustBundle(selectedPath);
        await loadKeyManagementStatus();
        return `${result.statusMessage} ${result.trustedSigningKeyFingerprint}`;
      });
    })();
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

  const rollbackMigrations = (databaseType: DatabaseType) => {
    if (!selectedProjectId) {
      return;
    }

    void runAction(`migration:rollback:${databaseType}`, async () => {
      const result = await rollbackProjectDatabaseMigrations(
        selectedProjectId,
        databaseType,
        rollbackSteps[databaseType],
      );

      return [
        result.statusMessage,
        result.rolledBackMigrations.length > 0
          ? `Rolled back: ${result.rolledBackMigrations.join(", ")}`
          : undefined,
      ]
        .filter(Boolean)
        .join(" ");
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

        <div className="flex flex-wrap gap-2">
          <Button
            disabled={actionKey === "schedule:runDue"}
            onClick={runScheduledBackups}
            variant="secondary"
          >
            Run due backups
          </Button>
          <Button
            disabled={actionKey === "scheduler:install"}
            onClick={installScheduler}
            variant="secondary"
          >
            Install OS scheduler
          </Button>
          <Button
            disabled={!schedulerStatus?.installed || actionKey === "scheduler:uninstall"}
            onClick={uninstallScheduler}
            variant="secondary"
          >
            Remove OS scheduler
          </Button>
          <Button
            disabled={actionKey === "trust:refresh"}
            onClick={() => void loadKeyManagementStatus()}
            variant="secondary"
          >
            Refresh keys
          </Button>
          <Button
            disabled={actionKey === "trust:export"}
            onClick={exportTrustBundle}
            variant="secondary"
          >
            Export trust bundle
          </Button>
          <Button
            disabled={actionKey === "trust:import"}
            onClick={importTrustBundle}
            variant="secondary"
          >
            Import trust bundle
          </Button>
        </div>
        {schedulerStatus ? (
          <p className="border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
            {schedulerStatus.statusMessage}
            {schedulerStatus.manifestPath ? ` Manifest: ${schedulerStatus.manifestPath}` : ""}
          </p>
        ) : null}
        {keyManagementStatus ? (
          <p className="border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
            {keyManagementStatus.statusMessage} Encryption:{" "}
            {keyManagementStatus.encryptionKeySource}. Signing:{" "}
            {keyManagementStatus.signingKeySource}. Trusted fingerprints:{" "}
            {keyManagementStatus.trustedSigningKeyFingerprints.length}.
            {keyManagementStatus.externalKmsProvider
              ? ` KMS: ${keyManagementStatus.externalKmsProvider}${
                  keyManagementStatus.externalKmsKeyId
                    ? ` / ${keyManagementStatus.externalKmsKeyId}`
                    : ""
                }.`
              : ""}
          </p>
        ) : null}
      </section>

      <section className="grid gap-4 xl:grid-cols-2">
        {databaseTypes.map((databaseType) => {
          const profile = profileByType[databaseType];
          const provisioningResult = provisioningResults[databaseType];
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

              {provisioningResult ? (
                <section className="grid gap-3 border border-voicebox-border bg-voicebox-surface p-3">
                  <div>
                    <p className="font-mono text-xs uppercase text-voicebox-tertiary">
                      Managed provisioning
                    </p>
                    <p className="mt-1 text-sm font-semibold text-voicebox-black">
                      {provisioningResult.statusMessage}
                    </p>
                  </div>

                  {provisioningResult.dependencyReport ? (
                    <div className="grid gap-2">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-mono text-xs uppercase text-voicebox-secondary">
                          {provisioningResult.dependencyReport.provider}
                        </span>
                        <span
                          className={dependencyClassName(
                            provisioningResult.dependencyReport.status,
                          )}
                        >
                          {provisioningResult.dependencyReport.status}
                        </span>
                      </div>
                      <div className="flex flex-wrap gap-2">
                        {provisioningResult.dependencyReport.packages.map((managedPackage) => (
                          <span
                            key={managedPackage.packageName}
                            className={packageClassName(managedPackage)}
                          >
                            {managedPackage.packageName}: {packageLabel(managedPackage)}
                          </span>
                        ))}
                      </div>
                    </div>
                  ) : null}

                  {provisioningResult.serviceReport ? (
                    <p className="font-mono text-xs leading-relaxed text-voicebox-secondary">
                      {provisioningResult.serviceReport.statusMessage}
                    </p>
                  ) : null}

                  {provisioningResult.phpmyadminAccess ? (
                    <div className="grid gap-2 border-t border-voicebox-border pt-3">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-mono text-xs uppercase text-voicebox-secondary">
                          phpMyAdmin
                        </span>
                        <span
                          className={
                            provisioningResult.phpmyadminAccess.reverseProxyStarted
                              ? "border border-voicebox-success px-2 py-1 font-mono text-xs uppercase text-voicebox-success"
                              : "border border-voicebox-warning px-2 py-1 font-mono text-xs uppercase text-voicebox-warning"
                          }
                        >
                          {provisioningResult.phpmyadminAccess.reverseProxyStarted
                            ? "Proxy requested"
                            : "Proxy pending"}
                        </span>
                      </div>
                      <a
                        className="break-all font-mono text-xs font-bold text-voicebox-black underline"
                        href={provisioningResult.phpmyadminAccess.url}
                        rel="noreferrer"
                        target="_blank"
                      >
                        {provisioningResult.phpmyadminAccess.url}
                      </a>
                      <p className="break-all font-mono text-xs text-voicebox-secondary">
                        Config: {provisioningResult.phpmyadminAccess.configPath}
                      </p>
                      <p className="break-all font-mono text-xs text-voicebox-secondary">
                        Proxy: {provisioningResult.phpmyadminAccess.reverseProxyConfigPath}
                      </p>
                    </div>
                  ) : null}
                </section>
              ) : null}

              <div className="flex flex-wrap gap-2">
                <Button
                  disabled={!selectedProjectId || actionKey === `provision:${databaseType}`}
                  onClick={() => provisionDatabase(databaseType)}
                >
                  Provision
                </Button>
                <Button
                  disabled={!ready || actionKey === `migration:run:${databaseType}`}
                  onClick={() => runMigrations(databaseType)}
                  variant="secondary"
                >
                  Run migrations
                </Button>
              </div>

              <DatabaseBackupControls
                actionKey={actionKey}
                backupOptions={backupOptions[databaseType]}
                databaseType={databaseType}
                policy={scheduleInputs[databaseType]}
                pointInTimeTarget={pointInTimeTargets[databaseType]}
                ready={ready}
                remoteDestination={remoteDestinationInputs[databaseType]}
                replaySourcePath={replaySourcePaths[databaseType]}
                rollbackSteps={rollbackSteps[databaseType]}
                restorePath={restorePaths[databaseType]}
                onBackup={() => backupDatabase(databaseType)}
                onBackupOptionsChange={(options) =>
                  setBackupOptions((currentOptions) => ({
                    ...currentOptions,
                    [databaseType]: options,
                  }))
                }
                onPickRestorePath={() => pickRestorePath(databaseType)}
                onPickReplaySourcePath={() => pickReplaySourcePath(databaseType)}
                onPolicyChange={(policy) =>
                  setScheduleInputs((currentInputs) => ({
                    ...currentInputs,
                    [databaseType]: policy,
                  }))
                }
                onPointInTimeRestore={() => restorePointInTime(databaseType)}
                onPointInTimeTargetChange={(target) =>
                  setPointInTimeTargets((currentTargets) => ({
                    ...currentTargets,
                    [databaseType]: target,
                  }))
                }
                onRemoteDestinationChange={(destination) =>
                  setRemoteDestinationInputs((currentInputs) => ({
                    ...currentInputs,
                    [databaseType]: destination,
                  }))
                }
                onRollbackMigrations={() => rollbackMigrations(databaseType)}
                onRollbackStepsChange={(steps) =>
                  setRollbackSteps((currentSteps) => ({
                    ...currentSteps,
                    [databaseType]: steps,
                  }))
                }
                onRestore={() => restoreDatabase(databaseType)}
                onReplayRestore={() => restoreWithReplay(databaseType)}
                onSaveRemoteDestination={() => saveBackupDestination(databaseType)}
                onSavePolicy={() => saveBackupPolicy(databaseType)}
              />

              {backupDestinations[databaseType] ? (
                <p className="border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
                  {remoteDestinationStatus(backupDestinations[databaseType])}
                </p>
              ) : null}

              {backupPolicies[databaseType] ? (
                <p className="border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
                  {scheduleStatus(backupPolicies[databaseType])}
                </p>
              ) : null}

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
                <div className="flex flex-wrap gap-2">
                  <Button
                    disabled={!profile}
                    onClick={() => pickMigrationForRollbackGeneration(databaseType)}
                    variant="secondary"
                  >
                    Pick migration
                  </Button>
                  <Button
                    disabled={
                      !profile ||
                      !rollbackGenerationPaths[databaseType].trim() ||
                      actionKey === `migration:generateRollback:${databaseType}`
                    }
                    onClick={() => generateRollbackSql(databaseType)}
                    variant="secondary"
                  >
                    Generate rollback
                  </Button>
                </div>
                {rollbackGenerationPaths[databaseType] ? (
                  <p className="break-all font-mono text-xs text-voicebox-secondary">
                    {rollbackGenerationPaths[databaseType]}
                  </p>
                ) : null}
                {rollbackGenerationResults[databaseType] ? (
                  <p className="break-all font-mono text-xs text-voicebox-secondary">
                    Rollback: {rollbackGenerationResults[databaseType]?.rollbackPath}
                  </p>
                ) : null}
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

function provisioningNotice(result: DatabaseProvisioningResult) {
  return [
    result.statusMessage,
    result.dependencyReport?.statusMessage,
    result.serviceReport?.statusMessage,
    result.phpmyadminAccess?.statusMessage,
  ]
    .filter(Boolean)
    .join(" ");
}

function dependencyClassName(status: ManagedDatabaseDependencyStatus) {
  const baseClassName = "border px-2 py-1 font-mono text-xs uppercase";

  if (status === "installed") {
    return `${baseClassName} border-voicebox-success text-voicebox-success`;
  }

  return `${baseClassName} border-voicebox-warning text-voicebox-warning`;
}

function packageClassName(managedPackage: ManagedDatabasePackage) {
  const baseClassName = "border px-2 py-1 font-mono text-xs uppercase";

  if (managedPackage.installedNow || managedPackage.alreadyInstalled) {
    return `${baseClassName} border-voicebox-success text-voicebox-success`;
  }

  return `${baseClassName} border-voicebox-warning text-voicebox-warning`;
}

function packageLabel(managedPackage: ManagedDatabasePackage) {
  if (managedPackage.installedNow) {
    return "installed";
  }

  if (managedPackage.alreadyInstalled) {
    return "present";
  }

  return "pending";
}

function backupOptionsFromPolicy(
  policy: DatabaseBackupPolicy | undefined,
  fallback: DatabaseBackupOptions,
): DatabaseBackupOptions {
  if (!policy) {
    return fallback;
  }

  return {
    compression: policy.compression,
    encryption: policy.encryption,
    retentionDays: policy.retentionDays,
  };
}

function policyInputFromPolicy(
  policy: DatabaseBackupPolicy | undefined,
  fallback: DatabaseBackupPolicyUpdate,
): DatabaseBackupPolicyUpdate {
  if (!policy) {
    return fallback;
  }

  return {
    compression: policy.compression,
    enabled: policy.enabled,
    encryption: policy.encryption,
    intervalMinutes: policy.intervalMinutes,
    retentionDays: policy.retentionDays,
  };
}

function remoteDestinationInputFromDestination(
  destination: DatabaseBackupRemoteDestination | undefined,
  fallback: DatabaseBackupRemoteDestinationUpdate,
): DatabaseBackupRemoteDestinationUpdate {
  if (!destination) {
    return fallback;
  }

  return {
    destinationPath: destination.destinationPath,
    enabled: destination.enabled,
    provider: destination.provider,
  };
}

function remoteDestinationStatus(destination: DatabaseBackupRemoteDestination | undefined) {
  if (!destination) {
    return "No remote backup destination saved.";
  }

  return [
    destination.enabled ? "Remote copy enabled." : "Remote copy disabled.",
    `Provider ${destination.provider}.`,
    `Destination ${destination.destinationPath || "not set"}.`,
    `Updated ${formatDate(destination.updatedAt)}.`,
  ].join(" ");
}

function scheduleStatus(policy: DatabaseBackupPolicy | undefined) {
  if (!policy) {
    return "No scheduled backup policy saved.";
  }

  const scheduleState = policy.enabled
    ? `enabled every ${policy.intervalMinutes} minute(s)`
    : "disabled";

  return [
    `Schedule ${scheduleState}.`,
    `Retention ${policy.retentionDays} day(s).`,
    `Compression ${policy.compression}.`,
    `Encryption ${policy.encryption}.`,
    policy.lastRunAt ? `Last run ${formatDate(policy.lastRunAt)}.` : "Last run pending.",
    policy.nextRunAt ? `Next run ${formatDate(policy.nextRunAt)}.` : "Next run not scheduled.",
  ].join(" ");
}

function scheduledBackupNotice(result: {
  readonly statusMessage: string;
  readonly errors: readonly string[];
}) {
  return [result.statusMessage, result.errors.length > 0 ? result.errors.join(" ") : undefined]
    .filter(Boolean)
    .join(" ");
}

function pointInTimeTargetToIso(value: string) {
  if (!value.trim()) {
    return undefined;
  }

  const target = new Date(value);
  if (Number.isNaN(target.getTime())) {
    return undefined;
  }

  return target.toISOString();
}
