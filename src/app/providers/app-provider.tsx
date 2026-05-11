import type { ReactNode } from "react";

import { MotionProvider } from "./motion-provider";
import { MuiProvider } from "./mui-provider";
import { QueryProvider } from "./query-provider";

interface AppProviderProps {
  children: ReactNode;
}

export function AppProvider({ children }: AppProviderProps) {
  return (
    <MuiProvider>
      <MotionProvider>
        <QueryProvider>{children}</QueryProvider>
      </MotionProvider>
    </MuiProvider>
  );
}
