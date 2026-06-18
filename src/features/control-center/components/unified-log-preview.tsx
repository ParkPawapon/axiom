import type { ControlCenterLogPreview } from "../types/control-center.types";

interface UnifiedLogPreviewProps {
  preview: ControlCenterLogPreview;
  onOpenLogs: () => void;
}

export function UnifiedLogPreview({ preview, onOpenLogs }: UnifiedLogPreviewProps) {
  return (
    <section className="border border-voicebox-border bg-white p-4">
      <div className="flex items-center justify-between gap-3">
        <h2 className="font-display text-2xl uppercase leading-none text-voicebox-black">Logs</h2>
        <button className="font-mono text-xs uppercase underline" onClick={onOpenLogs} type="button">
          Open Logs
        </button>
      </div>
      <div className="mt-4 grid max-h-56 gap-2 overflow-auto bg-voicebox-black p-3">
        {preview.entries.length > 0 ? (
          preview.entries.map((entry) => (
            <pre
              className="whitespace-pre-wrap break-words border border-neutral-700 bg-neutral-950 p-2 font-mono text-xs leading-relaxed text-white"
              key={entry.id}
            >
              {entry.message}
            </pre>
          ))
        ) : (
          <p className="p-2 font-mono text-xs text-white">{preview.statusMessage}</p>
        )}
      </div>
    </section>
  );
}
