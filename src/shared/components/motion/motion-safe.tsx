import type { ReactNode } from "react";
import { LazyMotion, domAnimation } from "framer-motion";

interface MotionSafeProps {
  children: ReactNode;
}

export function MotionSafe({ children }: MotionSafeProps) {
  return <LazyMotion features={domAnimation}>{children}</LazyMotion>;
}
