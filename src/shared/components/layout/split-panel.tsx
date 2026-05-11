import type { ReactNode } from "react";

interface SplitPanelProps {
  primary: ReactNode;
  secondary: ReactNode;
}

export function SplitPanel({ primary, secondary }: SplitPanelProps) {
  return (
    <div className="grid gap-4 lg:grid-cols-[1fr_20rem]">
      <div>{primary}</div>
      <aside>{secondary}</aside>
    </div>
  );
}
