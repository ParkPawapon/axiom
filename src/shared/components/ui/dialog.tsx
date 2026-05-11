import type { HTMLAttributes } from "react";

import { cn } from "../../lib/cn";

export function DialogPanel({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("border-2 border-voicebox-black bg-white p-6 shadow-none", className)}
      role="dialog"
      {...props}
    />
  );
}
