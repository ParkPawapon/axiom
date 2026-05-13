import type { ButtonHTMLAttributes, ReactNode } from "react";

import { cn } from "../../lib/cn";

interface ToolbarButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: ReactNode;
  isActive?: boolean;
}

export function ToolbarButton({
  children,
  className,
  isActive = false,
  type = "button",
  ...props
}: ToolbarButtonProps) {
  return (
    <button
      className={cn(
        "inline-flex h-10 items-center justify-center border px-3 text-sm font-bold transition-colors disabled:cursor-not-allowed disabled:text-voicebox-tertiary",
        isActive
          ? "border-voicebox-black bg-voicebox-black text-white"
          : "border-voicebox-border bg-white text-voicebox-black hover:border-voicebox-black",
        className,
      )}
      type={type}
      {...props}
    >
      {children}
    </button>
  );
}
