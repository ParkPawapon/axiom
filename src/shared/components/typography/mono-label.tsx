import type { HTMLAttributes, ReactNode } from "react";

import { cn } from "../../lib/cn";

interface MonoLabelProps extends HTMLAttributes<HTMLParagraphElement> {
  children: ReactNode;
}

export function MonoLabel({ children, className, ...props }: MonoLabelProps) {
  return (
    <p className={cn("font-mono text-xs uppercase text-voicebox-secondary", className)} {...props}>
      {children}
    </p>
  );
}
