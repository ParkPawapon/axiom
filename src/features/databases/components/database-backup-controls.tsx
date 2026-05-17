import { Button } from "../../../shared/components/ui/button";
import { Input } from "../../../shared/components/ui/input";
import { Select } from "../../../shared/components/ui/select";
import type {
  DatabaseBackupOptions,
  DatabaseBackupPolicyUpdate,
  DatabaseBackupRemoteDestinationUpdate,
  DatabaseType,
} from "../types/database.types";

interface DatabaseBackupControlsProps {
  actionKey?: string;
  backupOptions: DatabaseBackupOptions;
  databaseType: DatabaseType;
  policy: DatabaseBackupPolicyUpdate;
  pointInTimeTarget: string;
  ready: boolean;
  remoteDestination: DatabaseBackupRemoteDestinationUpdate;
  rollbackSteps: number;
  restorePath: string;
  onBackup: () => void;
  onBackupOptionsChange: (options: DatabaseBackupOptions) => void;
  onPickRestorePath: () => void;
  onPolicyChange: (policy: DatabaseBackupPolicyUpdate) => void;
  onPointInTimeRestore: () => void;
  onRemoteDestinationChange: (destination: DatabaseBackupRemoteDestinationUpdate) => void;
  onRollbackMigrations: () => void;
  onRollbackStepsChange: (steps: number) => void;
  onRestore: () => void;
  onPointInTimeTargetChange: (target: string) => void;
  onSaveRemoteDestination: () => void;
  onSavePolicy: () => void;
}

export function DatabaseBackupControls({
  actionKey,
  backupOptions,
  databaseType,
  onBackup,
  onBackupOptionsChange,
  onPickRestorePath,
  onPolicyChange,
  onPointInTimeRestore,
  onPointInTimeTargetChange,
  onRemoteDestinationChange,
  onRollbackMigrations,
  onRollbackStepsChange,
  onRestore,
  onSaveRemoteDestination,
  onSavePolicy,
  policy,
  pointInTimeTarget,
  ready,
  remoteDestination,
  rollbackSteps,
  restorePath,
}: DatabaseBackupControlsProps) {
  return (
    <section className="grid gap-3 border border-voicebox-border bg-voicebox-surface p-3">
      <div>
        <p className="font-mono text-xs uppercase text-voicebox-tertiary">Backup automation</p>
        <h3 className="mt-1 font-display text-xl uppercase leading-none text-voicebox-black">
          Retention / Encryption
        </h3>
      </div>

      <div className="grid gap-3 md:grid-cols-3">
        <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
          Compression
          <Select
            value={backupOptions.compression}
            onChange={(event) =>
              onBackupOptionsChange({
                ...backupOptions,
                compression: event.target.value as DatabaseBackupOptions["compression"],
              })
            }
          >
            <option value="gzip">Gzip</option>
            <option value="none">None</option>
          </Select>
        </label>
        <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
          Encryption
          <Select
            value={backupOptions.encryption}
            onChange={(event) =>
              onBackupOptionsChange({
                ...backupOptions,
                encryption: event.target.value as DatabaseBackupOptions["encryption"],
              })
            }
          >
            <option value="aes256Gcm">AES-256-GCM</option>
            <option value="none">None</option>
          </Select>
        </label>
        <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
          Retention days
          <Input
            min={1}
            max={365}
            type="number"
            value={backupOptions.retentionDays}
            onChange={(event) =>
              onBackupOptionsChange({
                ...backupOptions,
                retentionDays: Number(event.target.value),
              })
            }
          />
        </label>
      </div>

      <div className="flex flex-wrap gap-2">
        <Button
          disabled={!ready || actionKey === `backup:${databaseType}`}
          onClick={onBackup}
          variant="secondary"
        >
          Backup now
        </Button>
        <Button
          disabled={!ready || actionKey === `restore:${databaseType}`}
          onClick={onPickRestorePath}
          variant="secondary"
        >
          Pick restore file
        </Button>
        <Button
          disabled={!ready || !restorePath.trim() || actionKey === `restore:${databaseType}`}
          onClick={onRestore}
          variant="secondary"
        >
          Restore selected
        </Button>
      </div>

      {restorePath ? (
        <p className="break-all font-mono text-xs text-voicebox-secondary">{restorePath}</p>
      ) : null}

      <div className="grid gap-3 border-t border-voicebox-border pt-3">
        <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
          Point-in-time target
          <Input
            type="datetime-local"
            value={pointInTimeTarget}
            onChange={(event) => onPointInTimeTargetChange(event.target.value)}
          />
        </label>
        <Button
          disabled={
            !ready || !pointInTimeTarget.trim() || actionKey === `restore:pitr:${databaseType}`
          }
          onClick={onPointInTimeRestore}
          variant="secondary"
        >
          Restore point in time
        </Button>
      </div>

      <div className="grid gap-3 border-t border-voicebox-border pt-3">
        <label className="flex items-center gap-2 text-sm font-semibold text-voicebox-black">
          <input
            checked={remoteDestination.enabled}
            className="h-4 w-4 accent-voicebox-black"
            type="checkbox"
            onChange={(event) =>
              onRemoteDestinationChange({
                ...remoteDestination,
                enabled: event.target.checked,
              })
            }
          />
          Remote destination enabled
        </label>
        <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
          Mounted destination path
          <Input
            value={remoteDestination.destinationPath}
            onChange={(event) =>
              onRemoteDestinationChange({
                ...remoteDestination,
                destinationPath: event.target.value,
              })
            }
            placeholder="/Volumes/backups/axiomphp"
          />
        </label>
        <Button
          disabled={!ready || actionKey === `remote:save:${databaseType}`}
          onClick={onSaveRemoteDestination}
          variant="secondary"
        >
          Save destination
        </Button>
      </div>

      <div className="grid gap-3 border-t border-voicebox-border pt-3">
        <label className="flex items-center gap-2 text-sm font-semibold text-voicebox-black">
          <input
            checked={policy.enabled}
            className="h-4 w-4 accent-voicebox-black"
            type="checkbox"
            onChange={(event) => onPolicyChange({ ...policy, enabled: event.target.checked })}
          />
          Scheduled backup enabled
        </label>
        <div className="grid gap-3 md:grid-cols-2">
          <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
            Interval minutes
            <Input
              min={5}
              max={43200}
              type="number"
              value={policy.intervalMinutes}
              onChange={(event) =>
                onPolicyChange({ ...policy, intervalMinutes: Number(event.target.value) })
              }
            />
          </label>
          <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
            Retention days
            <Input
              min={1}
              max={365}
              type="number"
              value={policy.retentionDays}
              onChange={(event) =>
                onPolicyChange({ ...policy, retentionDays: Number(event.target.value) })
              }
            />
          </label>
        </div>
        <Button
          disabled={!ready || actionKey === `schedule:save:${databaseType}`}
          onClick={onSavePolicy}
          variant="secondary"
        >
          Save schedule
        </Button>
      </div>

      <div className="grid gap-3 border-t border-voicebox-border pt-3">
        <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
          Rollback steps
          <Input
            min={1}
            max={50}
            type="number"
            value={rollbackSteps}
            onChange={(event) => onRollbackStepsChange(Number(event.target.value))}
          />
        </label>
        <Button
          disabled={!ready || actionKey === `migration:rollback:${databaseType}`}
          onClick={onRollbackMigrations}
          variant="secondary"
        >
          Rollback latest migrations
        </Button>
      </div>
    </section>
  );
}
