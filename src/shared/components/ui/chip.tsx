import type { HTMLAttributes, ReactNode } from "react";

import { cn } from "../../lib/cn";

export type ChipTone = "neutral" | "success" | "warning" | "error";

interface ChipProps extends HTMLAttributes<HTMLSpanElement> {
  children: ReactNode;
  tone?: ChipTone;
}

const toneClassNames: Record<ChipTone, string> = {
  neutral: "border-voicebox-black text-voicebox-black",
  success: "border-voicebox-success text-voicebox-success",
  warning: "border-voicebox-warning text-voicebox-warning",
  error: "border-voicebox-red text-voicebox-red",
};

export function Chip({ children, className, tone = "neutral", ...props }: ChipProps) {
  return (
    <span
      className={cn(
        "inline-flex min-h-7 items-center border px-2 font-mono text-xs uppercase leading-none",
        toneClassNames[tone],
        className,
      )}
      {...props}
    >
      {children}
    </span>
  );
}
