import type { HTMLAttributes } from "react";

import { cn } from "../../lib/cn";

export function Card({ className, ...props }: HTMLAttributes<HTMLElement>) {
  return (
    <article
      className={cn("border border-voicebox-border bg-voicebox-surface p-4 shadow-none", className)}
      {...props}
    />
  );
}
