import type { HTMLAttributes } from "react";

import { cn } from "../../lib/cn";

export function MonoLabel({ className, ...props }: HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      className={cn("font-mono text-xs uppercase text-voicebox-secondary", className)}
      {...props}
    />
  );
}
