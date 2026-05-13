import type { ReactNode } from "react";
import { LazyMotion, domAnimation } from "framer-motion";

interface MotionProviderProps {
  children: ReactNode;
}

export function MotionProvider({ children }: MotionProviderProps) {
  return <LazyMotion features={domAnimation}>{children}</LazyMotion>;
}
