import type { HTMLAttributes } from "react";

import { cn } from "../../lib/cn";

export function Chip({ className, ...props }: HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      className={cn(
        "inline-flex items-center border border-voicebox-border bg-white px-2 py-1 text-xs font-bold uppercase tracking-normal text-voicebox-secondary",
        className,
      )}
      {...props}
    />
  );
}
