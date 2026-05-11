import type { HTMLAttributes } from "react";

import { cn } from "../../lib/cn";

export function DisplayHeading({ className, ...props }: HTMLAttributes<HTMLHeadingElement>) {
  return (
    <h1 className={cn("font-display text-4xl uppercase leading-none", className)} {...props} />
  );
}
