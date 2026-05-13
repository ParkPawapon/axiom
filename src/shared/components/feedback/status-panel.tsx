import type { ReactNode } from "react";

import { cn } from "../../lib/cn";

export type StatusPanelTone = "neutral" | "success" | "warning" | "error";

interface StatusPanelProps {
  title: string;
  children?: ReactNode;
  tone?: StatusPanelTone;
}

const toneClassNames: Record<StatusPanelTone, string> = {
  neutral: "border-voicebox-black text-voicebox-black",
  success: "border-voicebox-success text-voicebox-success",
  warning: "border-voicebox-warning text-voicebox-warning",
  error: "border-voicebox-red text-voicebox-red",
};

export function StatusPanel({ children, title, tone = "neutral" }: StatusPanelProps) {
  return (
    <section className={cn("border-2 bg-white p-4", toneClassNames[tone])}>
      <h2 className="font-display text-xl uppercase leading-none">{title}</h2>
      {children ? (
        <div className="mt-3 text-sm leading-relaxed text-voicebox-secondary">{children}</div>
      ) : null}
    </section>
  );
}
