import { useState } from "react";

import { Button } from "../../../shared/components/ui/button";
import type { LogEntry } from "../../logs/types/log.types";

interface LogLineProps {
  readonly entry: LogEntry;
}

export function LogLine({ entry }: LogLineProps) {
  const [copied, setCopied] = useState(false);

  async function copyLine() {
    await globalThis.navigator.clipboard?.writeText(entry.message);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1200);
  }

  return (
    <div className="grid gap-2 border-b border-voicebox-border py-2 md:grid-cols-[5rem_6rem_minmax(0,1fr)_5.5rem]">
      <span className="font-mono text-xs text-voicebox-tertiary">#{entry.lineNumber}</span>
      <span className="font-mono text-xs uppercase text-voicebox-secondary">{entry.level}</span>
      <pre className="min-w-0 whitespace-pre-wrap break-words font-mono text-xs leading-relaxed text-voicebox-black">
        {entry.message}
      </pre>
      <Button className="h-8 px-2 text-xs" onClick={() => void copyLine()} variant="ghost">
        {copied ? "Copied" : "Copy"}
      </Button>
    </div>
  );
}
