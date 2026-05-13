import type { ReactNode } from "react";

import { cn } from "../../lib/cn";

interface SplitPanelProps {
  primary: ReactNode;
  secondary: ReactNode;
  className?: string;
}

export function SplitPanel({ className, primary, secondary }: SplitPanelProps) {
  return (
    <div className={cn("grid gap-5 xl:grid-cols-[minmax(0,1fr)_24rem]", className)}>
      <div className="min-w-0">{primary}</div>
      <aside className="min-w-0">{secondary}</aside>
    </div>
  );
}
