import type { ReactNode } from "react";

import { useReducedMotion } from "../../hooks/use-reduced-motion";

interface RevealProps {
  children: ReactNode;
}

export function Reveal({ children }: RevealProps) {
  const shouldReduceMotion = useReducedMotion();

  if (shouldReduceMotion) {
    return <>{children}</>;
  }

  return <div className="motion-safe:animate-[reveal_180ms_ease-out]">{children}</div>;
}
