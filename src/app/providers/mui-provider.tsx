import type { ReactNode } from "react";
import { CssBaseline, ThemeProvider } from "@mui/material";

import { muiTheme } from "../../core/theme/mui-theme";

interface MuiProviderProps {
  children: ReactNode;
}

export function MuiProvider({ children }: MuiProviderProps) {
  return (
    <ThemeProvider theme={muiTheme}>
      <CssBaseline />
      {children}
    </ThemeProvider>
  );
}
