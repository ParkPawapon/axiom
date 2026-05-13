import type { ReactNode } from "react";
import { m, useReducedMotion } from "framer-motion";

interface RevealProps {
  children: ReactNode;
}

export function Reveal({ children }: RevealProps) {
  const shouldReduceMotion = useReducedMotion();

  if (shouldReduceMotion) {
    return <>{children}</>;
  }

  return (
    <m.div
      animate={{ opacity: 1, y: 0 }}
      initial={{ opacity: 0, y: 8 }}
      transition={{ duration: 0.18, ease: "easeOut" }}
    >
      {children}
    </m.div>
  );
}
