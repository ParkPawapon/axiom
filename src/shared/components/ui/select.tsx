import type { SelectHTMLAttributes } from "react";

import { cn } from "../../lib/cn";

export function Select({ className, ...props }: SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select
      className={cn(
        "h-11 border-2 border-voicebox-black bg-white px-3 text-sm text-voicebox-black disabled:border-voicebox-border",
        className,
      )}
      {...props}
    />
  );
}
