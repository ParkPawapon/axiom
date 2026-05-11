import type { ReactNode } from "react";

interface MotionSafeProps {
  children: ReactNode;
}

export function MotionSafe({ children }: MotionSafeProps) {
  return <>{children}</>;
}
