import type { ReactNode } from "react";

import { Reveal } from "./reveal";

interface BorderRevealProps {
  children: ReactNode;
}

export function BorderReveal({ children }: BorderRevealProps) {
  return (
    <Reveal>
      <div className="border-2 border-voicebox-black bg-white">{children}</div>
    </Reveal>
  );
}
