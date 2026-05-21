import { voiceboxTokens } from "../design-system/voicebox.tokens";

export const muiTheme = {
  name: "voicebox-light",
  colorScheme: "light",
  tokens: {
    palette: {
      primary: voiceboxTokens.color.primaryBlack,
      accent: voiceboxTokens.color.accentRed,
      background: voiceboxTokens.color.background,
      surface: voiceboxTokens.color.surface,
      textPrimary: voiceboxTokens.color.textPrimary,
      textSecondary: voiceboxTokens.color.textSecondary,
      divider: voiceboxTokens.color.borderMedium,
    },
    shape: {
      borderRadius: 0,
      boxShadow: "none",
    },
    typography: {
      body: voiceboxTokens.typography.body,
      display: voiceboxTokens.typography.display,
    },
  },
} as const;
