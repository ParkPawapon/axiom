import type { ReactNode } from "react";

interface BorderRevealProps {
  children: ReactNode;
}

export function BorderReveal({ children }: BorderRevealProps) {
  return <div className="border border-voicebox-border">{children}</div>;
}
