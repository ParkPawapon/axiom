import { Button } from "../../../shared/components/ui/button";
import { Input } from "../../../shared/components/ui/input";
import type { AuditLogReadResult } from "../types/security.types";

interface AuditLogPanelProps {
  auditLog?: AuditLogReadResult;
  isBusy: boolean;
  retentionDays: number;
  onRefresh: () => void;
  onRetentionDaysChange: (value: number) => void;
  onPrune: () => void;
}

export function AuditLogPanel({
  auditLog,
  isBusy,
  onPrune,
  onRefresh,
  onRetentionDaysChange,
  retentionDays,
}: AuditLogPanelProps) {
  return (
    <section className="grid gap-4 border-2 border-voicebox-black bg-white p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">Audit trail</p>
          <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
            Security Events
          </h2>
        </div>
        <Button disabled={isBusy} onClick={onRefresh} variant="secondary">
          Refresh
        </Button>
      </div>

      <div className="flex flex-wrap items-end gap-3 border border-voicebox-border bg-voicebox-surface p-3">
        <label className="grid gap-2 text-sm font-semibold text-voicebox-black">
          Retention days
          <Input
            min={1}
            max={365}
            type="number"
            value={retentionDays}
            onChange={(event) => onRetentionDaysChange(Number(event.target.value))}
          />
        </label>
        <Button disabled={isBusy} onClick={onPrune} variant="secondary">
          Apply retention
        </Button>
      </div>

      {auditLog ? (
        <div className="grid gap-3">
          <p className="break-all font-mono text-xs text-voicebox-secondary">
            {auditLog.statusMessage} {auditLog.logFile}
          </p>
          <div className="grid max-h-[28rem] gap-2 overflow-auto border border-voicebox-border bg-voicebox-surface p-2">
            {auditLog.entries.length > 0 ? (
              auditLog.entries.map((entry) => (
                <article className="grid gap-1 border border-voicebox-border bg-white p-3" key={entry.id}>
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <span className="font-mono text-xs uppercase text-voicebox-black">
                      {entry.operation}
                    </span>
                    <span className="font-mono text-xs uppercase text-voicebox-secondary">
                      {new Date(entry.timestamp).toLocaleString()}
                    </span>
                  </div>
                  <p className="text-sm font-semibold text-voicebox-black">{entry.message}</p>
                  <p className="break-all font-mono text-xs text-voicebox-secondary">
                    {entry.status} / {entry.resource}
                  </p>
                </article>
              ))
            ) : (
              <p className="p-3 text-sm text-voicebox-secondary">No security audit entries yet.</p>
            )}
          </div>
        </div>
      ) : null}
    </section>
  );
}
