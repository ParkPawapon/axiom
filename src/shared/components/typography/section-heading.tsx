import type { HTMLAttributes } from "react";

import { cn } from "../../lib/cn";

export function SectionHeading({ className, ...props }: HTMLAttributes<HTMLHeadingElement>) {
  return (
    <h2 className={cn("font-display text-2xl uppercase leading-tight", className)} {...props} />
  );
}
