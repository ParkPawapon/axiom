import type { ProjectLogSource } from "../types/log.types";

interface LogViewerProps {
  query: string;
  source?: ProjectLogSource;
}

export function LogViewer({ query, source }: LogViewerProps) {
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

  const metadataLines = [
    `Project: ${source.projectName}`,
    `State: ${source.processState}`,
    `Log file: ${source.logFile ?? "Created when the project process starts"}`,
    `Backend message: ${source.statusMessage}`,
  ].filter((line) => line.toLowerCase().includes(query.toLowerCase()));

  return (
    <section className="border-2 border-voicebox-black bg-white p-5">
      <div className="border-b border-voicebox-border pb-4">
        <p className="font-mono text-xs uppercase text-voicebox-secondary">Log Viewer</p>
        <h2 className="mt-1 font-display text-2xl uppercase leading-none text-voicebox-black">
          {source.projectName}
        </h2>
      </div>
      <pre className="mt-5 min-h-64 overflow-auto border border-voicebox-border bg-voicebox-black p-4 font-mono text-xs leading-relaxed text-white">
        {metadataLines.length > 0
          ? metadataLines.join("\n")
          : "No visible log metadata matches the current filter."}
      </pre>
      <p className="mt-4 border-l-2 border-voicebox-black pl-3 font-mono text-xs leading-relaxed text-voicebox-secondary">
        Streaming and file tailing are intentionally not implemented in this frontend-only pass. The
        UI only exposes backend process log metadata that already exists.
      </p>
    </section>
  );
}
