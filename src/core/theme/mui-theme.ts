import { createTheme } from "@mui/material/styles";

import { voiceboxTokens } from "../design-system/voicebox.tokens";

export const muiTheme = createTheme({
  palette: {
    mode: "light",
    primary: {
      main: voiceboxTokens.color.primaryBlack,
      contrastText: voiceboxTokens.color.background,
    },
    error: {
      main: voiceboxTokens.color.error,
    },
    success: {
      main: voiceboxTokens.color.success,
    },
    warning: {
      main: voiceboxTokens.color.warning,
    },
    background: {
      default: voiceboxTokens.color.background,
      paper: voiceboxTokens.color.surface,
    },
    text: {
      primary: voiceboxTokens.color.textPrimary,
      secondary: voiceboxTokens.color.textSecondary,
    },
    divider: voiceboxTokens.color.borderMedium,
  },
  typography: {
    fontFamily: voiceboxTokens.typography.body,
    h1: {
      fontFamily: voiceboxTokens.typography.display,
      letterSpacing: 0,
    },
    h2: {
      fontFamily: voiceboxTokens.typography.display,
      letterSpacing: 0,
    },
    button: {
      fontWeight: 700,
      textTransform: "none",
    },
  },
  shape: {
    borderRadius: 0,
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: 0,
          boxShadow: "none",
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: {
          borderRadius: 0,
          boxShadow: "none",
        },
      },
    },
  },
});
