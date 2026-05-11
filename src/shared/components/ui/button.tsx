import type { ButtonHTMLAttributes } from "react";

import { cn } from "../../lib/cn";

export type ButtonVariant = "primary" | "secondary" | "ghost";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
}

const variantClassNames: Record<ButtonVariant, string> = {
  primary: "border-2 border-voicebox-black bg-voicebox-black text-white",
  secondary: "border-2 border-voicebox-black bg-transparent text-voicebox-black",
  ghost: "border border-transparent bg-transparent text-voicebox-black",
};

export function Button({ className, type = "button", variant = "primary", ...props }: ButtonProps) {
  return (
    <button
      className={cn(
        "inline-flex h-11 items-center justify-center px-4 text-sm font-bold transition-colors disabled:cursor-not-allowed disabled:border-voicebox-border disabled:text-voicebox-tertiary",
        variantClassNames[variant],
        className,
      )}
      type={type}
      {...props}
    />
  );
}
