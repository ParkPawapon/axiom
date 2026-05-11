import type { InputHTMLAttributes } from "react";

import { cn } from "../../lib/cn";

export function Input({ className, ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      className={cn(
        "h-11 border-2 border-voicebox-black bg-white px-3 text-sm text-voicebox-black placeholder:text-voicebox-tertiary disabled:border-voicebox-border",
        className,
      )}
      {...props}
    />
  );
}
