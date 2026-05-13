import { Chip } from "../../../shared/components/ui/chip";
import type { LogLevel, ProjectLogReadResult, ProjectLogSource } from "../types/log.types";

interface LogViewerProps {
  result?: ProjectLogReadResult;
  source?: ProjectLogSource;
}

const levelTone: Record<LogLevel, "neutral" | "success" | "warning" | "error"> = {
  debug: "neutral",
  error: "error",
  info: "success",
  warn: "warning",
};

function formatFileSize(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }

  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }

  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function LogViewer({ result, source }: LogViewerProps) {
  if (!source) {
    return (
      <section className="border-2 border-voicebox-black bg-white p-5">
        <h2 className="font-display text-2xl uppercase leading-none text-voicebox-black">
          No Log Source
        </h2>
        <p className="mt-3 text-sm text-voicebox-secondary">
          Start a project process to create a backend-managed PHP process log file.
        </p>
      </section>
    );
  }

  return (
    <section className="border-2 border-voicebox-black bg-white p-5">
      <div className="border-b border-voicebox-border pb-4">
        <p className="font-mono text-xs uppercase text-voicebox-secondary">Log Viewer</p>
        <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
          {source.projectName}
        </h2>
        <div className="mt-3 flex flex-wrap gap-2">
          <Chip tone={source.processState === "running" ? "success" : "neutral"}>
            {source.processState}
          </Chip>
          {result ? <Chip>{result.returnedLines} lines</Chip> : null}
          {result?.truncated ? <Chip tone="warning">Tail view</Chip> : null}
        </div>
      </div>

      <div className="mt-5 grid max-h-[32rem] gap-2 overflow-auto border border-voicebox-border bg-voicebox-black p-3">
        {result?.entries.length ? (
          result.entries.map((entry) => (
            <article
              className="grid gap-2 border border-neutral-700 bg-neutral-950 p-3 text-white lg:grid-cols-[5rem_6rem_minmax(0,1fr)]"
              key={entry.id}
            >
              <span className="font-mono text-xs text-neutral-400">#{entry.lineNumber}</span>
              <Chip tone={levelTone[entry.level]}>{entry.level}</Chip>
              <pre className="min-w-0 whitespace-pre-wrap break-words font-mono text-xs leading-relaxed text-white">
                {entry.message}
              </pre>
            </article>
          ))
        ) : (
          <p className="p-3 font-mono text-xs text-white">
            {result?.statusMessage ?? "Select a project log source to read the log file."}
          </p>
        )}
      </div>

      <div className="mt-4 grid gap-2 border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
        <p>{result?.statusMessage ?? source.statusMessage}</p>
        <p className="break-words">
          Log file: {result?.logFile ?? source.logFile ?? "Created when the process starts"}
        </p>
        {result ? <p>File size: {formatFileSize(result.fileSizeBytes)}</p> : null}
      </div>
    </section>
  );
}
