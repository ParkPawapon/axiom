import type { HTMLAttributes, ReactNode } from "react";

import { cn } from "../../lib/cn";

interface DisplayHeadingProps extends HTMLAttributes<HTMLHeadingElement> {
  children: ReactNode;
}

export function DisplayHeading({ children, className, ...props }: DisplayHeadingProps) {
  return (
    <h1
      className={cn("font-display text-4xl uppercase leading-none text-voicebox-black", className)}
      {...props}
    >
      {children}
    </h1>
  );
}
