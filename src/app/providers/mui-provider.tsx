import type { ReactNode } from "react";

import { muiTheme } from "../../core/theme/mui-theme";

interface MuiProviderProps {
  children: ReactNode;
}

export function MuiProvider({ children }: MuiProviderProps) {
  return (
    <div data-mui-theme={muiTheme.name} style={{ colorScheme: muiTheme.colorScheme }}>
      {children}
    </div>
  );
}
