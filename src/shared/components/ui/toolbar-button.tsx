import type { ButtonHTMLAttributes } from "react";

import { cn } from "../../lib/cn";

export function ToolbarButton({
  className,
  type = "button",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      className={cn(
        "inline-flex h-9 items-center border border-voicebox-border bg-white px-3 text-xs font-bold text-voicebox-black disabled:text-voicebox-tertiary",
        className,
      )}
      type={type}
      {...props}
    />
  );
}
