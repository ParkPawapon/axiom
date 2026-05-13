import type { HTMLAttributes, ReactNode } from "react";

import { cn } from "../../lib/cn";

interface SectionHeadingProps extends HTMLAttributes<HTMLHeadingElement> {
  children: ReactNode;
}

export function SectionHeading({ children, className, ...props }: SectionHeadingProps) {
  return (
    <h2
      className={cn("font-display text-2xl uppercase leading-none text-voicebox-black", className)}
      {...props}
    >
      {children}
    </h2>
  );
}
