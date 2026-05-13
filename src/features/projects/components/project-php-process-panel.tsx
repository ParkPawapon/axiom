import { Button } from "../../../shared/components/ui/button";
import type { ProjectPhpProcessStatus } from "../types/project.types";

interface ProjectPhpProcessPanelProps {
  canStart: boolean;
  isBusy: boolean;
  status?: ProjectPhpProcessStatus;
  onRefresh: () => void;
  onStart: () => void;
  onStop: () => void;
}

const stateLabels: Record<ProjectPhpProcessStatus["state"], string> = {
  failed: "FAILED",
  running: "RUNNING",
  stopped: "STOPPED",
};

const stateClassNames: Record<ProjectPhpProcessStatus["state"], string> = {
  failed: "border-voicebox-red text-voicebox-red",
  running: "border-voicebox-success text-voicebox-success",
  stopped: "border-voicebox-border text-voicebox-secondary",
};

export function ProjectPhpProcessPanel({
  canStart,
  isBusy,
  onRefresh,
  onStart,
  onStop,
  status,
}: ProjectPhpProcessPanelProps) {
  const isRunning = status?.state === "running";

  return (
    <section className="border-2 border-voicebox-black bg-white p-5">
      <div className="flex flex-col gap-3 border-b border-voicebox-border pb-4 md:flex-row md:items-end md:justify-between">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">Project Process</p>
          <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
            PHP Server
          </h2>
        </div>
        {status ? (
          <span
            className={`inline-flex h-9 items-center border-2 px-3 font-mono text-xs uppercase ${stateClassNames[status.state]}`}
          >
            {stateLabels[status.state]}
          </span>
        ) : null}
      </div>

      <div className="mt-5 grid gap-3 md:grid-cols-2">
        <ProcessDetail label="URL" value={status?.url ?? "Not running"} />
        <ProcessDetail label="PID" value={status?.pid ? String(status.pid) : "None"} />
        <ProcessDetail label="PHP" value={status?.phpVersion ?? "No active process"} />
        <ProcessDetail label="Port" value={status?.port ? String(status.port) : "None"} />
        <ProcessDetail label="Document root" value={status?.documentRoot ?? "No active process"} />
        <ProcessDetail label="Log file" value={status?.logFile ?? "Created on start"} />
      </div>

      <p className="mt-5 border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
        {status?.statusMessage ?? "Process status has not loaded yet."}
      </p>

      <div className="mt-5 flex flex-wrap gap-2">
        <Button disabled={!canStart || isRunning || isBusy} onClick={onStart} variant="primary">
          {isBusy && !isRunning ? "Starting" : "Start"}
        </Button>
        <Button disabled={!isRunning || isBusy} onClick={onStop} variant="secondary">
          {isBusy && isRunning ? "Stopping" : "Stop"}
        </Button>
        <Button disabled={isBusy} onClick={onRefresh} variant="ghost">
          Refresh
        </Button>
      </div>
    </section>
  );
}

interface ProcessDetailProps {
  label: string;
  value: string;
}

function ProcessDetail({ label, value }: ProcessDetailProps) {
  return (
    <div className="border border-voicebox-border bg-voicebox-surface p-3">
      <p className="font-mono text-xs uppercase text-voicebox-secondary">{label}</p>
      <p className="mt-2 break-words font-mono text-xs text-voicebox-black">{value}</p>
    </div>
  );
}
